use super::*;
use std::collections::HashMap;

// Place outside src so that file writes do not trigger `cargo watch`.
static TEST_OUTPUT_DIR: &'static str = "/tmp";

fn get_test_file() -> String {
    let now: DateTime<Utc> = Utc::now();
    format!("{}/del-test-{:?}-{}.sqlite3", TEST_OUTPUT_DIR, now, get_unique_id())
}


// @todo/low Test strings with many queries are ignored: "BEGIN; SELECT 1".

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_default_pragmas() {
        let file = get_test_file();
        let c1 = DbFile::new(file.clone()).expect("Ok");

        // Note: `sqlite3_stmt_readonly` = false for `PRAGMA ...`
        let wtx1 = c1.get_write_tx().expect("Ok");


        let a = wtx1.q("PRAGMA journal_mode").expect("Ok");
        let b = wtx1.q("PRAGMA synchronous").expect("Ok");
        // Compile time flag `SQLITE_THREADSAFE=1` is checked at the Rust FFI layer.
        // (1, serialized) = Enforce correct concurrent usage by using all available mutexes.


        match (&a.rows.data[0][0], &b.rows.data[0][0]) {
            (
                Val::String(journal_mode),
                Val::I64(synchronous),
            ) => {
                assert_eq!(journal_mode, "wal");
                assert_eq!(*synchronous, 1); // 1 = Normal
            }
            _ => assert!(false)
        }
    }


    #[test]
    fn test_write_tx() {

        // Assert: Simple usage.
        {
            let file = get_test_file();

            let f = DbFile::new(file.clone()).expect("Ok");
            let wtx = f.get_write_tx().expect("Ok");


            wtx.q("CREATE TABLE t1(a PRIMARY KEY, b);").expect("Ok");


            assert!(wtx.q("INVALID COMMAND").is_err());

            let (f, result) = wtx.commit();
            assert!(result.is_ok());


            let wtx = f.get_write_tx().expect("Ok");
            wtx.q("INSERT INTO t1 (a, b) VALUES (1, 2), (3, 4)").expect("Ok");
            let (f, result) = wtx.commit();
            assert!(result.is_ok());


            let wtx = f.get_write_tx().expect("Ok");
            let RSet { num_cols, num_rows, .. } = wtx.q("SELECT * FROM t1").expect("Ok");
            assert_eq!((num_cols, num_rows), (2, 2));
            let (f, result) = wtx.commit();
            assert!(result.is_ok());
        }


        // Assert: Two concurrent write transactions, the second gets `SQLITE_BUSY`.
        {
            let file = get_test_file();

            let c1 = DbFile::new(file.clone()).expect("Ok");
            let c2 = DbFile::new(file.clone()).expect("Ok");

            let wtx1 = c1.get_write_tx().expect("Ok");

            let wtx2_result = c2.get_write_tx();
            assert!(wtx2_result.is_err());
            if let Err((c2, return_status)) = wtx2_result {
                assert_eq!(return_status.primary.id, PrimaryRC::SQLITE_BUSY);
            } else {
                assert!(false);
            }

            wtx1.q("CREATE TABLE t1(a PRIMARY KEY, b);").expect("Ok");
            wtx1.q("INSERT INTO t1 (a, b) VALUES (1, 2), (3, 4)").expect("Ok");
            let (f, result) = wtx1.commit();
            assert!(result.is_ok());
        }

        // Assert: Rollback works.
        {
            let file = get_test_file();

            let c1 = DbFile::new(file.clone()).expect("Ok");
            let c2 = DbFile::new(file.clone()).expect("Ok");

            let wtx1 = c1.get_write_tx().expect("Ok");
            wtx1.q("CREATE TABLE t1(a PRIMARY KEY, b);").expect("Ok");
            wtx1.q("INSERT INTO t1 (a, b) VALUES (1, 2), (3, 4)").expect("Ok");
            let (c1, res) = wtx1.rollback();
            assert!(res.is_ok());


            // Query fails because `create table` was rolled back.
            let wtx2 = c2.get_write_tx().expect("Ok");
            assert!(wtx2.q("SELECT * FROM t1").is_err());
            let (c2, res) = wtx2.rollback();
            assert!(res.is_ok());
        }


        // Assert: Params work, indexed.
        {
            let file = get_test_file();
            let c1 = DbFile::new(file.clone()).expect("Ok");

            let wtx1 = c1.get_write_tx().expect("Ok");
            wtx1.q("CREATE TABLE t1(a PRIMARY KEY, b);").expect("Ok");
            let p = Params::Index(vec![Val::I64(3), Val::I64(4)]);
            wtx1.q_params("INSERT INTO t1 (a, b) VALUES (1, 2), (?, ?)", &p).expect("Ok");
            let (c1, res) = wtx1.commit();
            assert!(res.is_ok());


            let wtx = c1.get_write_tx().expect("Ok");
            let RSet { num_cols, num_rows, rows, .. } = wtx.q("SELECT * FROM t1").expect("Ok");
            assert_eq!((num_cols, num_rows), (2, 2));

            match (&rows.data[1][0], &rows.data[1][1]) {
                (Val::I64(3), Val::I64(4)) => assert!(true),
                _ => assert!(false)
            }

            let (c1, result) = wtx.commit();
            assert!(result.is_ok());
        }

        // Assert: Params work, keyed.
        {
            let file = get_test_file();
            let c1 = DbFile::new(file.clone()).expect("Ok");

            let wtx1 = c1.get_write_tx().expect("Ok");
            wtx1.q("CREATE TABLE t1(a PRIMARY KEY, b);").expect("Ok");


            let data: HashMap<String, Val> = [
                ("x".to_string(), Val::I64(8)),
                ("y".to_string(), Val::I64(9)),
            ].iter().cloned().collect();
            let p = Params::Key(KeyVal { data });
            wtx1.q_params("INSERT INTO t1 (a, b) VALUES (1, 2), (:x, :y)", &p).expect("Ok");
            let (c1, res) = wtx1.commit();
            assert!(res.is_ok());


            let wtx = c1.get_write_tx().expect("Ok");
            let RSet { num_cols, num_rows, rows, .. } = wtx.q("SELECT * FROM t1").expect("Ok");
            assert_eq!((num_cols, num_rows), (2, 2));

            match (&rows.data[1][0], &rows.data[1][1]) {
                (Val::I64(8), Val::I64(9)) => assert!(true),
                _ => assert!(false)
            }

            let (c1, result) = wtx.commit();
            assert!(result.is_ok());
        }


        // Assert: One write transaction, many read transactions are OK.
        {
            let file = get_test_file();
            let c1 = DbFile::new(file.clone()).expect("Ok");
            let c2 = DbFile::new(file.clone()).expect("Ok");

            let c2 = create_table_a(c2).expect("Ok");
            let (c2, count) = row_count_close(c2);
            assert_eq!(count, 2);


            // Read lock creates a snapshot point, ignores any writes commited to `c2`.
            // @todo/low Error: "SQLITE_SCHEMA, statement aborts at 10: [SELECT 1 FROM sqlite_master LIMIT 0] database schema has changed"
            // Returned to log, NOT API.
            // This is due to `c2` changing the schema.
            let rtx1 = c1.get_read_tx().expect("Ok");

            let c2 = ins_row(c2, 10, 11);

            assert_eq!(row_count_r(&rtx1), 2);

            let c2 = ins_row(c2, 12, 13);
            let c2 = ins_row(c2, 14, 15);


            // Assert: `ReadTx` cannot see writes that occur after it began.
            assert_eq!(row_count_r(&rtx1), 2);
            let (c2, count) = row_count_close(c2);
            assert_eq!(count, 5);
        }
    }


    // Assert: When using the read tx API, only accept a SQL string that is read.
    #[test]
    fn test_read_tx_no_writes() {
        {
            let file = get_test_file();
            let c1 = DbFile::new(file.clone()).expect("Ok");
            let rtx1 = c1.get_read_tx().expect("Ok");

            let res = rtx1.q("CREATE TABLE t1(a PRIMARY KEY, b);");
            match res {
                Err(ReadError::QueryIsWrite) => assert!(true),
                _ => assert!(false)
            }
        }
    }

    // Assert: When using the write tx API, only accept a SQL string that matches the read/write function used.
    #[test]
    fn test_write_tx_enforce_category() {
        {
            let file = get_test_file();
            let c1 = DbFile::new(file.clone()).expect("Ok");
            let rtx1 = c1.get_write_tx().expect("Ok");

            let r1 = rtx1.q("CREATE TABLE t1(a PRIMARY KEY, b);");
            assert!(r1.is_ok());

            let (a, b, c) = (
                "CREATE TABLE t2(a PRIMARY KEY, b);",
                "INSERT INTO t1 (a, b) VALUES (1, 2)",
                "SELECT * FROM t1"
            );


            let r2 = rtx1.read(a);
            let r3 = rtx1.read(b);
            let r4 = rtx1.read(c);


            let r5 = rtx1.write(a);
            let r6 = rtx1.write(b);
            let r7 = rtx1.write(c);

            // @todo/low `params` API.

            match (r2, r3, r4, r5, r6, r7) {
                (
                    Err(ReadError::QueryIsWrite),
                    Err(ReadError::QueryIsWrite),
                    Ok(_),
                    Ok(_),
                    Ok(_),
                    Err(WriteError::QueryIsRead)
                ) => assert!(true),
                _ => assert!(false)
            }


            // `q` can be either read/write.
            let r8 = rtx1.q("INSERT INTO t1 (a, b) VALUES (3, 4)");
            let r9 = rtx1.q(c);

            match (r8, r9) {
                (
                    Ok(_),
                    Ok(_),
                ) => assert!(true),
                _ => assert!(false)
            }
        }
    }


}


fn create_table_a(c1: DbFile) -> Result<DbFile, ReturnStatus> {
    let wtx1 = c1.get_write_tx().expect("Ok");
    wtx1.q("CREATE TABLE t1(a PRIMARY KEY, b);").expect("Ok");
    let p = Params::Index(vec![Val::I64(3), Val::I64(4)]);
    wtx1.q_params("INSERT INTO t1 (a, b) VALUES (1, 2), (?, ?)", &p).expect("Ok");
    let (c1, res) = wtx1.commit();
    res?;

    Ok(c1)
}

fn ins_row(c1: DbFile, a: i64, b: i64) -> DbFile {
    let wtx1 = c1.get_write_tx().expect("Ok");
    let p = Params::Index(vec![Val::I64(a), Val::I64(b)]);
    wtx1.q_params("INSERT INTO t1 (a, b) VALUES (?, ?)", &p).expect("Ok");
    let (c1, res) = wtx1.commit();
    assert!(res.is_ok());
    c1
}


fn row_eq(rtx: &ReadTx, row: usize, vals: (i64, i64)) -> bool {
    let RSet { num_cols, num_rows, rows, .. } = rtx.q("SELECT * FROM t1").expect("Ok");
    assert_eq!((num_cols, num_rows), (2, 2));

    match (&rows.data[row][0], &rows.data[row][1]) {
        (Val::I64(a), Val::I64(b)) => *a == vals.0 && *b == vals.1,
        _ => false
    }
}


fn row_count_r(rtx: &ReadTx) -> i64 {
    let RSet { rows, .. } = rtx.q("SELECT count(*) as count FROM t1").expect("Ok");
    match rows.data[0][0] {
        Val::I64(v) => v,
        _ => {
            assert!(false);
            0
        }
    }
}

fn row_count_close(c1: DbFile) -> (DbFile, i64) {
    let rtx = c1.get_read_tx().expect("Ok");

    let RSet { rows, .. } = rtx.q("SELECT count(*) as count FROM t1").expect("Ok");
    let count = match rows.data[0][0] {
        Val::I64(v) => v,
        _ => {
            assert!(false);
            0
        }
    };

    let (c1, res) = rtx.commit();
    assert!(res.is_ok());

    (c1, count)
}


fn write_update_row(c1: DbFile) -> Result<DbFile, ReturnStatus> {
    let wtx1 = c1.get_write_tx().expect("Ok");
    wtx1.q("UP").expect("Ok");
    let p = Params::Index(vec![Val::I64(3), Val::I64(4)]);
    wtx1.q_params("INSERT INTO t1 (a, b) VALUES (1, 2), (?, ?)", &p).expect("Ok");
    let (c1, res) = wtx1.commit();
    res?;

    Ok(c1)
}