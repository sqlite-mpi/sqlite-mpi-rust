extern crate chrono;

use std::{thread, time};
use chrono::{DateTime, Utc};

use sqlite3ffi::errmap::PrimaryRC;
use sma::ReadError;


use super::*;
use crate::messages::*;

use test_envs::a::EnvA;

// Use library directly to check file state (instead of going through sma/runtime).
use sqlite3ffi::db::DbHandle;
use sqlite3ffi::stmt::Val;

// Place outside src so that file writes do not trigger `cargo watch`.
static TEST_OUTPUT_DIR: &'static str = "/tmp";
//static TEST_OUTPUT_DIR: &'static str = "/tmp";


fn get_test_file() -> String {
    let now: DateTime<Utc> = Utc::now();
    // format!("{}/del-test-{:?}-{}.sqlite3", TEST_OUTPUT_DIR, now, get_unique_id())
    format!("{}/del-test-{:?}.sqlite3", TEST_OUTPUT_DIR, now)
}


// @todo/important Move env creation functions to a different module; define tests in high level interactions only.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::InMsg::*;
    use crate::messages::FileOp::*;
    use crate::simulator::*;


    fn integrity(f: String) {
        assert!(integrity_check(f));
    }

    // Assert: Read transaction can be created.
    #[test]
    fn test_runtime_read_tx() {
        let get = get_new_runtime();

        let fa = "/tmp/a.sqlite".to_string();
        let fb = "/tmp/b.sqlite".to_string();

        let req_a = File(
            GetReadTx(
                ArgsFile {
                    file: fa.clone()
                }
            )
        );
        let req_b = File(
            GetReadTx(
                ArgsFile {
                    file: fb.clone()
                }
            )
        );

        let a = get(req_a);
        let b = get(req_b);

        match (&a, &b) {
            (OutMsg::File(Ok(_)), OutMsg::File(Ok(_))) => assert!(true),
            _ => assert!(false)
        };


        // Assert: Read works in read tx,
        if let OutMsg::File(Ok(TxIdOnly { tx_id })) = &a {
            let in_a = get_tx_q(&tx_id, "SELECT * FROM sqlite_master");
            let out_a = get(in_a);

            let in_b = get_tx_q(&tx_id, "CREATE TABLE t1(a PRIMARY KEY, b);");
            let out_b = get(in_b);

            match out_b {
                OutMsg::Tx(Err(TxOpErr::ReadError(ReadError::QueryIsWrite))) => assert!(true),
                _ => assert!(false)
            };
        }

        integrity(fa);
        integrity(fb);
    }


    // Assert: Read and write transaction isolation, write request queue.
    #[test]
    fn test_runtime_read_and_write_tx() {
        let s1 = |e: &EnvA| {
            e.w1_write_group_a();
            e.r1_cannot_see_group_a();
            e.w2_no_response_yet();
        };


        // Assert: `commit` works.
        {
            let f = get_test_file();
            let mut e = EnvA::new(f.clone());
            s1(&e);

            e.w1_commit();
            e.w2_has_response();

            e.r1_cannot_see_group_a();
            e.w2_sees_group_a();
            e.r2_sees_group_a();

            integrity(f);
        }


        // Assert: `rollback` works.
        {
            let f = get_test_file();
            let mut e = EnvA::new(f.clone());
            s1(&e);

            e.w1_rollback();
            e.w2_has_response();

            e.r1_cannot_see_group_a();
            e.w2_cannot_see_group_a();

            integrity(f);
        }
        // @todo/low Assert: Read and write tx will timeout after a period of no interaction.


        // @todo/low Ensure db connections are closed; No `-wal` or `-shm` files exist.
        // `PRAGMA wal_checkpoint` returns (0, 0, 0) after reopening the file.
    }


    // Assert: Transactions work as expected when other processes on the OS are writing to the same file.
    // Assert: `SQLITE_BUSY` is handled correctly and predictably.
    // - Uses many threads to simulate multiple processes on an OS.
    #[test]
    fn test_multiple_concurrent_runtimes() {


        // Assert: Many concurrent run times will queue writes, but allow concurrent reads.
        {

            // Same file; reads and writes from multiple concurrent runtimes will contend for locks.
            // - Write locks in particular will implicitly queue; when the db file is locked the other runtimes will get `SQLITE_BUSY`
            // - Write transaction requests will queue per runtime, but the write at the start of each runtime queue will compete (poll) for a SQLite write lock.
            let f = get_test_file();


            let mut all_t = vec![];
            for n in 0..8 {
                let d = time::Duration::from_millis(1);
                thread::sleep(d);

                let f = f.clone();

                let t = thread::spawn(move || {
                    println!("test thread id: {:?}", thread::current().id());
                    let (i, o) = get_new_runtime_async();

                    let i_id = writes(&i, &f, 1).pop().unwrap();
                    let tx_id = get_tx_id(get_single(&o, &i_id));
                    q(&i, &o, &tx_id, &"CREATE TABLE IF NOT EXISTS wtx (id INTEGER PRIMARY KEY, tx_id, req_at, started_at, diff, ins_at);".to_string());
                    commit(&i, &o, &tx_id);
                    let num_inserts = 1;

                    for n in 0..num_inserts {
                        let req_at: DateTime<Utc> = Utc::now();
                        let w = writes(&i, &f, 1);

                        // Assumption: wtx request order == process order.
                        for i_id in w {
                            let tx_id = get_tx_id(get_single(&o, &i_id));
                            let started_at: DateTime<Utc> = Utc::now();
                            let diff = started_at - req_at;
                            let s = format!("INSERT INTO wtx (tx_id, req_at, started_at, diff, ins_at) VALUES ('{}', '{}', '{}', '{}', (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')))", tx_id, req_at, started_at, diff);
                            q(&i, &o, &tx_id, &s);
                            commit(&i, &o, &tx_id);
                        }
                    }

                    num_inserts
                });
                all_t.push(t);
            }


            // Wait for all threads to complete.
            let mut total_inserts = 0;
            for t in all_t {
                total_inserts += t.join().expect("Ok");
            }

            integrity(f.clone());


            // Assert: File contains correct number of rows.
            let rows = DbHandle::new(f.clone()).unwrap().run("SELECT count(*) as count FROM wtx").unwrap().rows.data;
            if let Val::I64(count) = rows[0][0] {
                assert_eq!(total_inserts, count);
            } else {
                assert!(false);
            }
        }
    }


    // Assert: Permission denied returned from SQLite FFI.
    #[test]
    fn test_runtime_read_tx_err() {
        let get = get_new_runtime();

        let req_a = File(
            GetReadTx(
                ArgsFile {
                    file: "/a.sqlite".to_string()
                }
            )
        );

        let a = get(req_a);

        match &a {
            OutMsg::File(Err(e)) => {
                match &e {
                    FileOpErr::ReturnStatus(rs) => {
                        assert_eq!(rs.primary.id, PrimaryRC::SQLITE_CANTOPEN);
                    }
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }
}
