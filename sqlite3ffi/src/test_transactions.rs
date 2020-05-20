use super::*;
use placeholder::PlaceholderTypes;
use serde_json::json;
use crate::stmt::ErrorBind;

// Place outside src so that file writes do not trigger `cargo watch`.
static TEST_OUTPUT_DIR: &'static str = "/tmp";

fn get_test_file() -> String {
    let now: DateTime<Utc> = Utc::now();
    format!("{}/del-test-{:?}.sqlite3", TEST_OUTPUT_DIR, now)
}

fn run(h: &DbHandle, q: &str) -> RSet {
    let s = StmtHandle::new(&h, q).expect("Syntax Ok");
    s.run().expect("Run ok")
}

fn run_bind(h: &DbHandle, q: &str, v: &Vec<Val>) {}

#[test]
fn test_transaction_basic_ok() {
    let file = get_test_file();
    let A = &DbHandle::new(file.clone()).unwrap();
    let B = &DbHandle::new(file.clone()).unwrap();

    // @see https://www.sqlite.org/threadsafe.html


    // Question: Should the transaction state be stored in Rust, or read from SQLite?
    // Question: Create functions that enforce semantics, or allow client/user to BEGIN/COMMIT them selves?
    // - Can Rust ownership help here?

    // @see https://www.sqlite.org/c3ref/busy_timeout.html
    // @see https://www.sqlite.org/c3ref/busy_handler.html
    // @see https://www.sqlite.org/wal.html
    // @see https://www.sqlite.org/cgi/src/doc/begin-concurrent/doc/begin_concurrent.md
    // Principle: choose your state machine.


    // @todo/next Get `PRAGMA journal_mode`, assert its correct (delete, wal).
    // @todo/next https://www.sqlite.org/pragma.html#pragma_synchronous, set to normal mode, default is full?
    // Allow user to change via an API?
    // Question: Should the user be allowed to change this setting? How does it impact concurrency?
    {
        dbg!(run(A, "PRAGMA journal_mode=WAL"));
        dbg!(run(B, "PRAGMA journal_mode=WAL"));

//        run(A, "SELECT 1");
//        run(B, "SELECT 1");
//
//
//        run(A, "BEGIN");
//        run(A, "CREATE TABLE t1(a PRIMARY KEY, b);");
//
//        run(B, "BEGIN");
//        run(B, "CREATE TABLE t1(a PRIMARY KEY, b);");
//
//        println!("a");
//        run(B, "COMMIT");
//
//        run(A, "COMMIT");


        // Issue: When is a read transaction started?
        // - @see http://mailinglists.sqlite.org/cgi-bin/mailman/private/sqlite-users/2019-July/085284.html

        run(A, "CREATE TABLE t1(a PRIMARY KEY, b);");

        run(B, "BEGIN");
//        dbg!(run(B, "SELECT * FROM t1"));
//        dbg!(run(B, "SELECT 1"));

        run(A, "BEGIN");
        dbg!(run(A, "INSERT INTO t1 (a, b) VALUES (1, 2), (3, 4)"));

//        dbg!(run(B, "SELECT * FROM t1"));
        dbg!(run(A, "INSERT INTO t1 (a, b) VALUES (5, 6), (7, 8)"));
        run(A, "COMMIT");

        dbg!(run(B, "SELECT * FROM t1"));


        // @see https://sqlite.org/lang_transaction.html#immediate
        // FULL, IOERR, BUSY, NOMEM = ROLLBACK
        // - "It is recommended that applications respond to the errors listed above by explicitly issuing a ROLLBACK command. If the transaction has already been rolled back automatically by the error response, then the ROLLBACK command will fail with an error, but no harm is caused by this."


        // Read vs write transaction
        // BEGIN SELECT COMMIT. This reads a snapshot ignoring all writers.
        // BEGIN SELECT UPDATE COMMIT. Starts a read transaction, moves to a write transaction *will fail with SQLITE_BUSY_SNAPSHOT if a writer has written those update records before the commit.
        // BEGIN IMMEDIATE Starts a write transaction, will not return SQLITE_BUSY for any statements.


        // @see https://www.sqlite.org/isolation.html
        // - There is no isolation for operations on the same connection.
        //      - Transactions are per connection.
        // = One connection for reads, one for writes?
    }


    // @todo/important Test extended error codes.
}
