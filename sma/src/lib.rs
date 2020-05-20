#![allow(warnings)]

// sma = State Machine A
// - "Choose your own state machine".
//      - Instead of giving the user 100 config options with conflicting runtime characteristics, allow them to pick from a small amount of state machines.
// - Allow the user to choose the runtime mode that suits them.
// - Enable developing the next API separately (smb, smc etc).
// - Re-use the same low level SQLite primitives, but expose different levels of public API on top.
// - This layer enables adding a serialization/messaging layer on top.
//      - E.g. Objects can be referenced by a string ID instead of a C pointer like in a FFI.
//      - Works optimally by default with SQLite concurrency restrictions.
// - Prevent having to support every mode or variant in a single code base.
//      - Enables clear documentation, as there is only one possible behavior regardless of config.

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use std::thread;

extern crate chrono;

use chrono::{DateTime, Utc};


pub mod fns;

use fns::get_unique_id;


#[cfg(test)]
mod test;


use sqlite3ffi::{
    db::{
        DbHandle,
        BindRunError,
    },
    stmt::{
        RSet,
        Rows,
        Val,
        KeyVal,
        IndexVal,
        ErrorBind,
    },
    err::ReturnStatus,
    errmap::{
        PrimaryRow,
        PrimaryRC,
        ExtendedRC,
    },
};
use std::hint::unreachable_unchecked;

#[derive(Debug)]
pub struct DbFile {
    id: String,
    db_handle: DbHandle,
}

#[derive(Debug)]
pub struct WriteTx {
    pub id: String,
    db_file: DbFile,
}

#[derive(Debug)]
pub struct ReadTx {
    pub id: String,
    db_file: DbFile,
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum Params {
    #[serde(rename = "key_based")]
    Key(KeyVal),

    #[serde(rename = "index_based")]
    Index(IndexVal),
}

#[derive(Debug)]
enum JournalMode {
    Delete,
    Truncate,
    Persist,
    Memory,
    WAL,
    Off,
}

fn ms(i: u64) -> Duration {
    Duration::from_millis(i)
}

impl JournalMode {
    pub fn new(mode: &str) -> JournalMode {
        match mode {
            "delete" => JournalMode::Delete,
            "truncate" => JournalMode::Truncate,
            "persist" => JournalMode::Persist,
            "memory" => JournalMode::Memory,
            "wal" => JournalMode::WAL,
            "off" => JournalMode::Off,
            _ => unreachable!("Unknown PRAGMA journal_mode = {}", mode)
        }
    }

    pub fn from_db(db_handle: &DbHandle) -> Result<JournalMode, ReturnStatus> {
        // @todo/low move to "db_handle.get_pragma"?
        match db_handle.run("PRAGMA journal_mode")? {
            RSet {
                rows: Rows { mut data, .. },
                ..
            } => {
                if let Some(mut rw) = data.pop() {
                    if let Some(Val::String(mode)) = rw.pop() {
                        return Ok(JournalMode::new(&mode));
                    }
                }
            }
            _ => unreachable!("Could not read `PRAGMA journal_mode`")
        }
        unreachable!("Incorrect rows returned for `PRAGMA journal_mode`")
    }
}


impl<'a> DbFile {
    pub fn new(file: String) -> Result<DbFile, ReturnStatus> {
        let r = DbHandle::new(file);


        let db_handle = match r {
            Err(e) => {
                dbg!(("Error when getting db handle", &e));
                return Err(e);
            }
            Ok(db) => db
        };


        DbFile::set_connection_pragmas(&db_handle)?;

        Ok(
            DbFile {
                id: get_unique_id(),
                db_handle,
            }
        )
    }

    // @todo/low Are these consistent across db handles (once set they are persisted in the db file)?
    // @todo/low Is it better to set these options via a SQLite compile flag?
    // @todo/low Block changing concurrency PRAGMAs so there is only one state machine for the above layers?
    // Allow customization via options?
    fn set_connection_pragmas(db_handle: &DbHandle) -> Result<(), ReturnStatus> {
        DbFile::set_journal_mode_wal(db_handle)?;

        // Note: unlike `PRAGMA journal_mode=WAL`, this does not need a write lock, and only applies per connection.

        // @see https://www.sqlite.org/pragma.html#pragma_synchronous
        // "NORMAL" = Transactions unsafe if OS crashes (but safe when the application process fails).
        // "WAL mode is safe from corruption with synchronous=NORMAL"
        // "The synchronous=NORMAL setting is a good choice for most applications running in WAL mode."
        match db_handle.run("PRAGMA synchronous=NORMAL") {
            Err(e) => {
                dbg!(("Error when setting `PRAGMA synchronous=NORMAL`", &e));
                return Err(e);
            }
            _ => {}
        }

        Ok(())
    }


    // Force WAL mode.
    // - Levels built on top assume WAL mode (e.g threading, queueing and ownership).
    // - Its the newer mode; supports concurrent reads, single writer.
    //      - Other modes provide no benefit when used with an event loop.
    // - Prevent confusing people with lots of different modes/configs/APIs.
    //
    // WAL mode:
    // - Requires a write lock to set.
    // - Applies per file (is persistent after the connection closes).
    // @see https://www.sqlite.org/wal.html
    fn set_journal_mode_wal(db_handle: &DbHandle) -> Result<(), ReturnStatus> {
        let t = ms(16);

        let wal_enabled = loop {
            let jmode = JournalMode::from_db(&db_handle);

            let jmode = match jmode {
                Err(e) => {
                    dbg!("Error when *reading* `PRAGMA journal_mode`");
                    dbg!(&e);

                    if e.primary.id == PrimaryRC::SQLITE_BUSY {
                        if let Some(_) = e.extended {
                            // Issue: `SQLITE_BUSY_RECOVERY` is returned when many threads write to the same file concurrently.
                            // @todo/important does this indicate invalid uses of connection/stmt pointers?
                            // @todo/important Prove correct pointer usage: Convert DTrace scripts to eBPF, run on Linux, visualize with Interplay.
                            dbg!("Extended error code returned when reading `PRAGMA journal_mode`.");
                        }

                        thread::sleep(t);
                        continue;
                    }

                    // E.g `SQLITE_READONLY`
                    return Err(e);
                }
                Ok(m) => m
            };


//            dbg!((&jmode, thread::current().id()));

            match jmode {
                JournalMode::WAL => break true,
                _ => {
                    match db_handle.run("PRAGMA journal_mode=WAL") {
                        Err(ReturnStatus { primary: PrimaryRow { id: PrimaryRC::SQLITE_BUSY, .. }, .. }) => {
                            // This should only happen on the first file open.
                            thread::sleep(t);
                            continue;
                        }
                        Err(e) => {
                            dbg!(&e);
                            unreachable!("Unexpected error when *writing* `PRAGMA journal_mode=WAL`. SQLite return code = {:?}", e.primary.id);
                            break false;
                        }
                        Ok(_) => continue
                    }
                }
            }
        };

        assert!(wal_enabled);
        Ok(())
    }


    pub fn get_read_tx(self) -> Result<ReadTx, (DbFile, ReturnStatus)> {
        // Issue: There is no `BEGIN IMMEDIATE` counterpart for a read transaction.
        // - `BEGIN` does nothing until the first read/write query.
        // - The first read acquires a `SHARED` (read) lock which forces subsequent read queries inside the transaction to read from the same snapshot point.
        let rset = self.db_handle.run("BEGIN;");

        if let Err(e) = rset {
            return Err((self, e));
        }

        // @todo/medium When two connections to a single file are held, and one mutates the schema, the second to select from `sqlite_master`
        // to get a read lock results in the below error being output to the log *but not returned in the FFI API*.
        // This could be an issue when all read locks return warnings to the log, but not in the API.
        // Error: "SQLITE_SCHEMA, statement aborts at 11: [SELECT * FROM sqlite_master] database schema has changed"
        let rset2 = self.db_handle.run("SELECT 1 FROM sqlite_master LIMIT 0");
        // Note: `SELECT 1` does not get a read lock.

        if let Err(e) = rset2 {
            return Err((self, e));
        }

        Ok(
            ReadTx {
                id: get_unique_id(),
                db_file: self,
            }
        )
    }

    // Converts `DbFile` into a `WriteTx`.
    pub fn get_write_tx(self) -> Result<WriteTx, (DbFile, ReturnStatus)> {
        let rset = self.db_handle.run("BEGIN IMMEDIATE");

        if let Err(e) = rset {
            return Err((self, e));
        }

        Ok(
            WriteTx {
                id: get_unique_id(),
                db_file: self,
            }
        )
    }

    // Assumption: `DbFile::new` is always called with the normalized file path.
    pub fn get_file_abs(&self) -> String {
        self.db_handle.file.clone()
    }

    // @todo/low Single queries,  make sure these route to correct read/write queues.
    // pub fn read() -> Result<RSet, ReturnStatus> {}
    // pub fn write() -> Result<RSet, ReturnStatus> {}
    // pub fn q() -> Result<RSet, ReturnStatus> {}
}

// @todo/low Should the read/write functions be traits?
// E.g. You want to pass either a `WriteTx` or `ReadTx` to a function where it will use the same APIs to read, but you do not want to have to declare an `enum` to group both types.


impl<'a> ReadTx {
    pub fn q(&self, q: &str) -> Result<RSet, ReadError> {
        let h = &self.db_file.db_handle;
        run_read_only(h, q)
    }

    pub fn q_params(&self, q: &str, p: &Params) -> Result<RSet, ReadBindRunError> {
        let h = &self.db_file.db_handle;
        run_params_read_only(h, q, p)
    }

    pub fn commit(self) -> (DbFile, Result<RSet, ReturnStatus>) {
        // @todo/medium In what ways can this fail? Should a fail prevent transfer of ownership and allow a re-try?
        let r = self.db_file.db_handle.run("COMMIT");

        // Take ownership of self, drop it. Return ownership of `db_file` to calling scope.
        // Assert: Only one transaction per connection should be active.
        // Its not possible to have multiple isolated transactions per file connection in SQLite.
        (self.db_file, r)
    }

    // Returns with "cannot rollback - no transaction is active" if already rolled back.
    pub fn rollback(self) -> (DbFile, Result<RSet, ReturnStatus>) {
        let r = self.db_file.db_handle.run("ROLLBACK");
        (self.db_file, r)
    }
}


#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum ReadError {
    QueryIsWrite,
    ReturnStatus(ReturnStatus),
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum ReadBindRunError {
    QueryIsWrite,
    BindRunError(BindRunError),
}


impl From<ReturnStatus> for ReadError {
    fn from(rs: ReturnStatus) -> Self {
        ReadError::ReturnStatus(rs)
    }
}

impl From<BindRunError> for ReadBindRunError {
    fn from(bre: BindRunError) -> Self {
        ReadBindRunError::BindRunError(bre)
    }
}

// @todo/low Can this be inferred by Rust automatically?
// E.g. If every `From` child -> parent is implemented, can descendant -> ancestor be inferred?
// `impl` scopes?
impl From<ReturnStatus> for ReadBindRunError {
    fn from(rs: ReturnStatus) -> Self {
        ReadBindRunError::BindRunError(BindRunError::ReturnStatus(rs))
    }
}

impl From<ErrorBind> for ReadBindRunError {
    fn from(eb: ErrorBind) -> Self {
        ReadBindRunError::BindRunError(BindRunError::ErrorBind(eb))
    }
}


#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum WriteError {
    QueryIsRead,
    ReturnStatus(ReturnStatus),
}


#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum WriteBindRunError {
    QueryIsRead,
    BindRunError(BindRunError),
}


impl From<ReturnStatus> for WriteError {
    fn from(rs: ReturnStatus) -> Self {
        WriteError::ReturnStatus(rs)
    }
}

impl From<BindRunError> for WriteBindRunError {
    fn from(bre: BindRunError) -> Self {
        WriteBindRunError::BindRunError(bre)
    }
}

impl From<ReturnStatus> for WriteBindRunError {
    fn from(rs: ReturnStatus) -> Self {
        WriteBindRunError::BindRunError(BindRunError::ReturnStatus(rs))
    }
}

impl From<ErrorBind> for WriteBindRunError {
    fn from(eb: ErrorBind) -> Self {
        WriteBindRunError::BindRunError(BindRunError::ErrorBind(eb))
    }
}


// Question: Why have separate read/write functions, instead of one general "query" function?
// Answer: SQL strings are black boxes to the host language.
// This enables flexibility of an external DSL, but also requires more mental work from the end user.
// Having the user categorise them into read/write sets encodes this meta data into the host languages AST.
// This enables AST based tools to analyse "where does the next state transition get written", OR "show me all writes in my code".
// Reads and writes have different concurrency attributes. This will could enable future runtime optimizations.
// `q` Is provided for read or writes, in case the end user does not want/need to use categorisation.
impl<'a> WriteTx {
    pub fn read(&self, q: &str) -> Result<RSet, ReadError> {
        // @todo/next test read only
        let h = &self.db_file.db_handle;
        run_read_only(h, q)
    }

    pub fn read_params(&self, q: &str, p: &Params) -> Result<RSet, ReadBindRunError> {
        let h = &self.db_file.db_handle;
        run_params_read_only(h, q, p)
    }

    pub fn write(&self, q: &str) -> Result<RSet, WriteError> {
        // @todo/low confirm only write
        let h = &self.db_file.db_handle;
        run_write_only(h, q)
    }

    pub fn write_params(&self, q: &str, p: &Params) -> Result<RSet, WriteBindRunError> {
        // @todo/low confirm only write
        let h = &self.db_file.db_handle;
        run_params_write_only(h, q, p)
    }

    pub fn q(&self, q: &str) -> Result<RSet, ReturnStatus> {
        self.db_file.db_handle.run(&q)
    }

    pub fn q_params(&self, q: &str, p: &Params) -> Result<RSet, BindRunError> {
        run_params(&self.db_file.db_handle, &q, &p)
    }


    pub fn commit(self) -> (DbFile, Result<RSet, ReturnStatus>) {
        // @todo/medium In what ways can this fail? Should a fail prevent transfer of ownership and allow a re-try?
        let r = self.q("COMMIT");

        // Take ownership of self, drop it. Return ownership of `db_file` to calling scope.
        // Assert: Only one transaction per connection should be active.
        // Its not possible to have multiple isolated transactions per file connection in SQLite.
        (self.db_file, r)
    }

    // Returns with "cannot rollback - no transaction is active" if already rolled back.
    pub fn rollback(self) -> (DbFile, Result<RSet, ReturnStatus>) {
        let r = self.q("ROLLBACK");
        (self.db_file, r)
    }

    // @todo/low What about `ROLLBACK TO SAVEPOINT` support?
}


// @todo/important Does an auto rollback/commit need to be done when a `ReadTx` or `WriteTx` goes out of scope?
// error[E0509]: cannot move out of type `WriteTx<'_>`, which implements the `Drop` trait
// Cannot move a structs field if that struct implements drop, as the field is no longer accessible.

//impl Drop for WriteTx<'_> {
//    fn drop(&mut self) {
//        // - Are transactions held as long as the connection is open?
//        if self.is_open {
//            let rb = self.rollback();
//            dbg!("Rollback called in drop");
//            dbg!(rb);
//        }
//    }
//}


fn run_params(db_handle: &DbHandle, q: &str, p: &Params) -> Result<RSet, BindRunError> {
    match p {
        Params::Key(kv) => {
            db_handle.run_kv(&q, &kv)
        }
        Params::Index(i) => {
            db_handle.run_index(&q, &i)
        }
    }
}


fn run_read_only(h: &DbHandle, q: &str) -> Result<RSet, ReadError> {
    let s = h.new_stmt(&q)?;

    if !s.is_read_only {
        return Err(ReadError::QueryIsWrite);
    }

    Ok(s.run()?)
}

fn run_params_read_only(h: &DbHandle, q: &str, p: &Params) -> Result<RSet, ReadBindRunError> {
    let s = h.new_stmt(&q)?;

    if !s.is_read_only {
        return Err(ReadBindRunError::QueryIsWrite);
    }

    match p {
        Params::Key(kv) => {
            &s.bind_kv(&kv)?;
        }
        Params::Index(i) => {
            &s.bind_index(&i)?;
        }
    }

    Ok(s.run()?)
}


fn run_write_only(h: &DbHandle, q: &str) -> Result<RSet, WriteError> {
    let s = h.new_stmt(&q)?;

    if s.is_read_only {
        return Err(WriteError::QueryIsRead);
    }

    Ok(s.run()?)
}

fn run_params_write_only(h: &DbHandle, q: &str, p: &Params) -> Result<RSet, WriteBindRunError> {
    let s = h.new_stmt(&q)?;

    if s.is_read_only {
        return Err(WriteBindRunError::QueryIsRead);
    }

    match p {
        Params::Key(kv) => {
            &s.bind_kv(&kv)?;
        }
        Params::Index(i) => {
            &s.bind_index(&i)?;
        }
    }

    Ok(s.run()?)
}
