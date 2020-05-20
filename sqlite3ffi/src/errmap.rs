// @todo/low
// - Map SQLite to Rust `Result` type?
// - Research: binding.rs defines return codes as `u32`, but the SQLite docs says they are `i32`, despite being all positive.

use serde::{Deserialize, Serialize};

use crate::cffi::{

    // Primary
    SQLITE_OK,
    SQLITE_ERROR,
    SQLITE_INTERNAL,
    SQLITE_PERM,
    SQLITE_ABORT,
    SQLITE_BUSY,
    SQLITE_LOCKED,
    SQLITE_NOMEM,
    SQLITE_READONLY,
    SQLITE_INTERRUPT,
    SQLITE_IOERR,
    SQLITE_CORRUPT,
    SQLITE_NOTFOUND,
    SQLITE_FULL,
    SQLITE_CANTOPEN,
    SQLITE_PROTOCOL,
    SQLITE_EMPTY,
    SQLITE_SCHEMA,
    SQLITE_TOOBIG,
    SQLITE_CONSTRAINT,
    SQLITE_MISMATCH,
    SQLITE_MISUSE,
    SQLITE_NOLFS,
    SQLITE_AUTH,
    SQLITE_FORMAT,
    SQLITE_RANGE,
    SQLITE_NOTADB,
    SQLITE_NOTICE,
    SQLITE_WARNING,
    SQLITE_ROW,
    SQLITE_DONE,

    // Extended
    SQLITE_ERROR_MISSING_COLLSEQ,
    SQLITE_ERROR_RETRY,
    SQLITE_ERROR_SNAPSHOT,
    SQLITE_IOERR_READ,
    SQLITE_IOERR_SHORT_READ,
    SQLITE_IOERR_WRITE,
    SQLITE_IOERR_FSYNC,
    SQLITE_IOERR_DIR_FSYNC,
    SQLITE_IOERR_TRUNCATE,
    SQLITE_IOERR_FSTAT,
    SQLITE_IOERR_UNLOCK,
    SQLITE_IOERR_RDLOCK,
    SQLITE_IOERR_DELETE,
    SQLITE_IOERR_BLOCKED,
    SQLITE_IOERR_NOMEM,
    SQLITE_IOERR_ACCESS,
    SQLITE_IOERR_CHECKRESERVEDLOCK,
    SQLITE_IOERR_LOCK,
    SQLITE_IOERR_CLOSE,
    SQLITE_IOERR_DIR_CLOSE,
    SQLITE_IOERR_SHMOPEN,
    SQLITE_IOERR_SHMSIZE,
    SQLITE_IOERR_SHMLOCK,
    SQLITE_IOERR_SHMMAP,
    SQLITE_IOERR_SEEK,
    SQLITE_IOERR_DELETE_NOENT,
    SQLITE_IOERR_MMAP,
    SQLITE_IOERR_GETTEMPPATH,
    SQLITE_IOERR_CONVPATH,
    SQLITE_IOERR_VNODE,
    SQLITE_IOERR_AUTH,
    SQLITE_IOERR_BEGIN_ATOMIC,
    SQLITE_IOERR_COMMIT_ATOMIC,
    SQLITE_IOERR_ROLLBACK_ATOMIC,
    SQLITE_LOCKED_SHAREDCACHE,
    SQLITE_LOCKED_VTAB,
    SQLITE_BUSY_RECOVERY,
    SQLITE_BUSY_SNAPSHOT,
    SQLITE_CANTOPEN_NOTEMPDIR,
    SQLITE_CANTOPEN_ISDIR,
    SQLITE_CANTOPEN_FULLPATH,
    SQLITE_CANTOPEN_CONVPATH,
    SQLITE_CANTOPEN_DIRTYWAL,
    SQLITE_CORRUPT_VTAB,
    SQLITE_CORRUPT_SEQUENCE,
    SQLITE_READONLY_RECOVERY,
    SQLITE_READONLY_CANTLOCK,
    SQLITE_READONLY_ROLLBACK,
    SQLITE_READONLY_DBMOVED,
    SQLITE_READONLY_CANTINIT,
    SQLITE_READONLY_DIRECTORY,
    SQLITE_ABORT_ROLLBACK,
    SQLITE_CONSTRAINT_CHECK,
    SQLITE_CONSTRAINT_COMMITHOOK,
    SQLITE_CONSTRAINT_FOREIGNKEY,
    SQLITE_CONSTRAINT_FUNCTION,
    SQLITE_CONSTRAINT_NOTNULL,
    SQLITE_CONSTRAINT_PRIMARYKEY,
    SQLITE_CONSTRAINT_TRIGGER,
    SQLITE_CONSTRAINT_UNIQUE,
    SQLITE_CONSTRAINT_VTAB,
    SQLITE_CONSTRAINT_ROWID,
    SQLITE_NOTICE_RECOVER_WAL,
    SQLITE_NOTICE_RECOVER_ROLLBACK,
    SQLITE_WARNING_AUTOINDEX,
    SQLITE_AUTH_USER,
    SQLITE_OK_LOAD_PERMANENTLY,
};


// RC =  Return Code.
// @see https://www.sqlite.org/c3ref/c_abort.html
#[derive(Debug)]
#[derive(PartialEq)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum PrimaryRC {
    SQLITE_OK,
    SQLITE_ERROR,
    SQLITE_INTERNAL,
    SQLITE_PERM,
    SQLITE_ABORT,
    SQLITE_BUSY,
    SQLITE_LOCKED,
    SQLITE_NOMEM,
    SQLITE_READONLY,
    SQLITE_INTERRUPT,
    SQLITE_IOERR,
    SQLITE_CORRUPT,
    SQLITE_NOTFOUND,
    SQLITE_FULL,
    SQLITE_CANTOPEN,
    SQLITE_PROTOCOL,
    SQLITE_EMPTY,
    SQLITE_SCHEMA,
    SQLITE_TOOBIG,
    SQLITE_CONSTRAINT,
    SQLITE_MISMATCH,
    SQLITE_MISUSE,
    SQLITE_NOLFS,
    SQLITE_AUTH,
    SQLITE_FORMAT,
    SQLITE_RANGE,
    SQLITE_NOTADB,
    SQLITE_NOTICE,
    SQLITE_WARNING,
    SQLITE_ROW,
    SQLITE_DONE,
}

// @see https://www.sqlite.org/c3ref/c_abort_rollback.html
// @todo/low Create enum tree based on primary group. E.g. SQLITE_XXXXXX_YYYYYY
#[derive(Debug)]
#[derive(PartialEq)]
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum ExtendedRC {
    SQLITE_ERROR_MISSING_COLLSEQ,
    SQLITE_ERROR_RETRY,
    SQLITE_ERROR_SNAPSHOT,
    SQLITE_IOERR_READ,
    SQLITE_IOERR_SHORT_READ,
    SQLITE_IOERR_WRITE,
    SQLITE_IOERR_FSYNC,
    SQLITE_IOERR_DIR_FSYNC,
    SQLITE_IOERR_TRUNCATE,
    SQLITE_IOERR_FSTAT,
    SQLITE_IOERR_UNLOCK,
    SQLITE_IOERR_RDLOCK,
    SQLITE_IOERR_DELETE,
    SQLITE_IOERR_BLOCKED,
    SQLITE_IOERR_NOMEM,
    SQLITE_IOERR_ACCESS,
    SQLITE_IOERR_CHECKRESERVEDLOCK,
    SQLITE_IOERR_LOCK,
    SQLITE_IOERR_CLOSE,
    SQLITE_IOERR_DIR_CLOSE,
    SQLITE_IOERR_SHMOPEN,
    SQLITE_IOERR_SHMSIZE,
    SQLITE_IOERR_SHMLOCK,
    SQLITE_IOERR_SHMMAP,
    SQLITE_IOERR_SEEK,
    SQLITE_IOERR_DELETE_NOENT,
    SQLITE_IOERR_MMAP,
    SQLITE_IOERR_GETTEMPPATH,
    SQLITE_IOERR_CONVPATH,
    SQLITE_IOERR_VNODE,
    SQLITE_IOERR_AUTH,
    SQLITE_IOERR_BEGIN_ATOMIC,
    SQLITE_IOERR_COMMIT_ATOMIC,
    SQLITE_IOERR_ROLLBACK_ATOMIC,
    SQLITE_LOCKED_SHAREDCACHE,
    SQLITE_LOCKED_VTAB,
    SQLITE_BUSY_RECOVERY,
    SQLITE_BUSY_SNAPSHOT,
    SQLITE_CANTOPEN_NOTEMPDIR,
    SQLITE_CANTOPEN_ISDIR,
    SQLITE_CANTOPEN_FULLPATH,
    SQLITE_CANTOPEN_CONVPATH,
    SQLITE_CANTOPEN_DIRTYWAL,
    SQLITE_CORRUPT_VTAB,
    SQLITE_CORRUPT_SEQUENCE,
    SQLITE_READONLY_RECOVERY,
    SQLITE_READONLY_CANTLOCK,
    SQLITE_READONLY_ROLLBACK,
    SQLITE_READONLY_DBMOVED,
    SQLITE_READONLY_CANTINIT,
    SQLITE_READONLY_DIRECTORY,
    SQLITE_ABORT_ROLLBACK,
    SQLITE_CONSTRAINT_CHECK,
    SQLITE_CONSTRAINT_COMMITHOOK,
    SQLITE_CONSTRAINT_FOREIGNKEY,
    SQLITE_CONSTRAINT_FUNCTION,
    SQLITE_CONSTRAINT_NOTNULL,
    SQLITE_CONSTRAINT_PRIMARYKEY,
    SQLITE_CONSTRAINT_TRIGGER,
    SQLITE_CONSTRAINT_UNIQUE,
    SQLITE_CONSTRAINT_VTAB,
    SQLITE_CONSTRAINT_ROWID,
    SQLITE_NOTICE_RECOVER_WAL,
    SQLITE_NOTICE_RECOVER_ROLLBACK,
    SQLITE_WARNING_AUTOINDEX,
    SQLITE_AUTH_USER,
    SQLITE_OK_LOAD_PERMANENTLY,
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct PrimaryRow {
    pub id: PrimaryRC,
    pub code: u32,
//    pub //str: String,
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct ExtendedRow {
    pub id: ExtendedRC,
    pub code: u32,
//    pub //str: String,
}

// Enable:
// - Using Rusts enum type checking to check all error variants are dealt with.
// - Define/encode (enum, intCode, strCode) mapping in one place.
// - (enum, intCode, strCode) = (Rust type system, SQLite FFI, User/logging)
const PRIMARYTBL: &[PrimaryRow] = &[
    PrimaryRow {
        id: PrimaryRC::SQLITE_OK,
        code: SQLITE_OK,
        //str: "SQLITE_OK".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_ERROR,
        code: SQLITE_ERROR,
        //str: "SQLITE_ERROR".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_INTERNAL,
        code: SQLITE_INTERNAL,
        //str: "SQLITE_INTERNAL".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_PERM,
        code: SQLITE_PERM,
        //str: "SQLITE_PERM".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_ABORT,
        code: SQLITE_ABORT,
        //str: "SQLITE_ABORT".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_BUSY,
        code: SQLITE_BUSY,
        //str: "SQLITE_BUSY".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_LOCKED,
        code: SQLITE_LOCKED,
        //str: "SQLITE_LOCKED".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_NOMEM,
        code: SQLITE_NOMEM,
        //str: "SQLITE_NOMEM".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_READONLY,
        code: SQLITE_READONLY,
        //str: "SQLITE_READONLY".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_INTERRUPT,
        code: SQLITE_INTERRUPT,
        //str: "SQLITE_INTERRUPT".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_IOERR,
        code: SQLITE_IOERR,
        //str: "SQLITE_IOERR".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_CORRUPT,
        code: SQLITE_CORRUPT,
        //str: "SQLITE_CORRUPT".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_NOTFOUND,
        code: SQLITE_NOTFOUND,
        //str: "SQLITE_NOTFOUND".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_FULL,
        code: SQLITE_FULL,
        //str: "SQLITE_FULL".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_CANTOPEN,
        code: SQLITE_CANTOPEN,
        //str: "SQLITE_CANTOPEN".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_PROTOCOL,
        code: SQLITE_PROTOCOL,
        //str: "SQLITE_PROTOCOL".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_EMPTY,
        code: SQLITE_EMPTY,
        //str: "SQLITE_EMPTY".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_SCHEMA,
        code: SQLITE_SCHEMA,
        //str: "SQLITE_SCHEMA".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_TOOBIG,
        code: SQLITE_TOOBIG,
        //str: "SQLITE_TOOBIG".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_CONSTRAINT,
        code: SQLITE_CONSTRAINT,
        //str: "SQLITE_CONSTRAINT".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_MISMATCH,
        code: SQLITE_MISMATCH,
        //str: "SQLITE_MISMATCH".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_MISUSE,
        code: SQLITE_MISUSE,
        //str: "SQLITE_MISUSE".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_NOLFS,
        code: SQLITE_NOLFS,
        //str: "SQLITE_NOLFS".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_AUTH,
        code: SQLITE_AUTH,
        //str: "SQLITE_AUTH".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_FORMAT,
        code: SQLITE_FORMAT,
        //str: "SQLITE_FORMAT".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_RANGE,
        code: SQLITE_RANGE,
        //str: "SQLITE_RANGE".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_NOTADB,
        code: SQLITE_NOTADB,
        //str: "SQLITE_NOTADB".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_NOTICE,
        code: SQLITE_NOTICE,
        //str: "SQLITE_NOTICE".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_WARNING,
        code: SQLITE_WARNING,
        //str: "SQLITE_WARNING".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_ROW,
        code: SQLITE_ROW,
        //str: "SQLITE_ROW".to_string(),
    },
    PrimaryRow {
        id: PrimaryRC::SQLITE_DONE,
        code: SQLITE_DONE,
        //str: "SQLITE_DONE".to_string(),
    },
];


static EXTENDENDEDTBL: &'static [ExtendedRow] = &[
    ExtendedRow {
        id: ExtendedRC::SQLITE_ERROR_MISSING_COLLSEQ,
        code: SQLITE_ERROR_MISSING_COLLSEQ,
        //str: "SQLITE_ERROR_MISSING_COLLSEQ".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_ERROR_RETRY,
        code: SQLITE_ERROR_RETRY,
        //str: "SQLITE_ERROR_RETRY".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_ERROR_SNAPSHOT,
        code: SQLITE_ERROR_SNAPSHOT,
        //str: "SQLITE_ERROR_SNAPSHOT".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_READ,
        code: SQLITE_IOERR_READ,
        //str: "SQLITE_IOERR_READ".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_SHORT_READ,
        code: SQLITE_IOERR_SHORT_READ,
        //str: "SQLITE_IOERR_SHORT_READ".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_WRITE,
        code: SQLITE_IOERR_WRITE,
        //str: "SQLITE_IOERR_WRITE".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_FSYNC,
        code: SQLITE_IOERR_FSYNC,
        //str: "SQLITE_IOERR_FSYNC".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_DIR_FSYNC,
        code: SQLITE_IOERR_DIR_FSYNC,
        //str: "SQLITE_IOERR_DIR_FSYNC".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_TRUNCATE,
        code: SQLITE_IOERR_TRUNCATE,
        //str: "SQLITE_IOERR_TRUNCATE".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_FSTAT,
        code: SQLITE_IOERR_FSTAT,
        //str: "SQLITE_IOERR_FSTAT".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_UNLOCK,
        code: SQLITE_IOERR_UNLOCK,
        //str: "SQLITE_IOERR_UNLOCK".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_RDLOCK,
        code: SQLITE_IOERR_RDLOCK,
        //str: "SQLITE_IOERR_RDLOCK".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_DELETE,
        code: SQLITE_IOERR_DELETE,
        //str: "SQLITE_IOERR_DELETE".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_BLOCKED,
        code: SQLITE_IOERR_BLOCKED,
        //str: "SQLITE_IOERR_BLOCKED".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_NOMEM,
        code: SQLITE_IOERR_NOMEM,
        //str: "SQLITE_IOERR_NOMEM".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_ACCESS,
        code: SQLITE_IOERR_ACCESS,
        //str: "SQLITE_IOERR_ACCESS".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_CHECKRESERVEDLOCK,
        code: SQLITE_IOERR_CHECKRESERVEDLOCK,
        //str: "SQLITE_IOERR_CHECKRESERVEDLOCK".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_LOCK,
        code: SQLITE_IOERR_LOCK,
        //str: "SQLITE_IOERR_LOCK".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_CLOSE,
        code: SQLITE_IOERR_CLOSE,
        //str: "SQLITE_IOERR_CLOSE".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_DIR_CLOSE,
        code: SQLITE_IOERR_DIR_CLOSE,
        //str: "SQLITE_IOERR_DIR_CLOSE".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_SHMOPEN,
        code: SQLITE_IOERR_SHMOPEN,
        //str: "SQLITE_IOERR_SHMOPEN".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_SHMSIZE,
        code: SQLITE_IOERR_SHMSIZE,
        //str: "SQLITE_IOERR_SHMSIZE".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_SHMLOCK,
        code: SQLITE_IOERR_SHMLOCK,
        //str: "SQLITE_IOERR_SHMLOCK".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_SHMMAP,
        code: SQLITE_IOERR_SHMMAP,
        //str: "SQLITE_IOERR_SHMMAP".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_SEEK,
        code: SQLITE_IOERR_SEEK,
        //str: "SQLITE_IOERR_SEEK".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_DELETE_NOENT,
        code: SQLITE_IOERR_DELETE_NOENT,
        //str: "SQLITE_IOERR_DELETE_NOENT".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_MMAP,
        code: SQLITE_IOERR_MMAP,
        //str: "SQLITE_IOERR_MMAP".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_GETTEMPPATH,
        code: SQLITE_IOERR_GETTEMPPATH,
        //str: "SQLITE_IOERR_GETTEMPPATH".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_CONVPATH,
        code: SQLITE_IOERR_CONVPATH,
        //str: "SQLITE_IOERR_CONVPATH".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_VNODE,
        code: SQLITE_IOERR_VNODE,
        //str: "SQLITE_IOERR_VNODE".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_AUTH,
        code: SQLITE_IOERR_AUTH,
        //str: "SQLITE_IOERR_AUTH".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_BEGIN_ATOMIC,
        code: SQLITE_IOERR_BEGIN_ATOMIC,
        //str: "SQLITE_IOERR_BEGIN_ATOMIC".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_COMMIT_ATOMIC,
        code: SQLITE_IOERR_COMMIT_ATOMIC,
        //str: "SQLITE_IOERR_COMMIT_ATOMIC".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_IOERR_ROLLBACK_ATOMIC,
        code: SQLITE_IOERR_ROLLBACK_ATOMIC,
        //str: "SQLITE_IOERR_ROLLBACK_ATOMIC".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_LOCKED_SHAREDCACHE,
        code: SQLITE_LOCKED_SHAREDCACHE,
        //str: "SQLITE_LOCKED_SHAREDCACHE".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_LOCKED_VTAB,
        code: SQLITE_LOCKED_VTAB,
        //str: "SQLITE_LOCKED_VTAB".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_BUSY_RECOVERY,
        code: SQLITE_BUSY_RECOVERY,
        //str: "SQLITE_BUSY_RECOVERY".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_BUSY_SNAPSHOT,
        code: SQLITE_BUSY_SNAPSHOT,
        //str: "SQLITE_BUSY_SNAPSHOT".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CANTOPEN_NOTEMPDIR,
        code: SQLITE_CANTOPEN_NOTEMPDIR,
        //str: "SQLITE_CANTOPEN_NOTEMPDIR".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CANTOPEN_ISDIR,
        code: SQLITE_CANTOPEN_ISDIR,
        //str: "SQLITE_CANTOPEN_ISDIR".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CANTOPEN_FULLPATH,
        code: SQLITE_CANTOPEN_FULLPATH,
        //str: "SQLITE_CANTOPEN_FULLPATH".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CANTOPEN_CONVPATH,
        code: SQLITE_CANTOPEN_CONVPATH,
        //str: "SQLITE_CANTOPEN_CONVPATH".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CANTOPEN_DIRTYWAL,
        code: SQLITE_CANTOPEN_DIRTYWAL,
        //str: "SQLITE_CANTOPEN_DIRTYWAL".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CORRUPT_VTAB,
        code: SQLITE_CORRUPT_VTAB,
        //str: "SQLITE_CORRUPT_VTAB".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CORRUPT_SEQUENCE,
        code: SQLITE_CORRUPT_SEQUENCE,
        //str: "SQLITE_CORRUPT_SEQUENCE".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_READONLY_RECOVERY,
        code: SQLITE_READONLY_RECOVERY,
        //str: "SQLITE_READONLY_RECOVERY".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_READONLY_CANTLOCK,
        code: SQLITE_READONLY_CANTLOCK,
        //str: "SQLITE_READONLY_CANTLOCK".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_READONLY_ROLLBACK,
        code: SQLITE_READONLY_ROLLBACK,
        //str: "SQLITE_READONLY_ROLLBACK".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_READONLY_DBMOVED,
        code: SQLITE_READONLY_DBMOVED,
        //str: "SQLITE_READONLY_DBMOVED".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_READONLY_CANTINIT,
        code: SQLITE_READONLY_CANTINIT,
        //str: "SQLITE_READONLY_CANTINIT".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_READONLY_DIRECTORY,
        code: SQLITE_READONLY_DIRECTORY,
        //str: "SQLITE_READONLY_DIRECTORY".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_ABORT_ROLLBACK,
        code: SQLITE_ABORT_ROLLBACK,
        //str: "SQLITE_ABORT_ROLLBACK".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CONSTRAINT_CHECK,
        code: SQLITE_CONSTRAINT_CHECK,
        //str: "SQLITE_CONSTRAINT_CHECK".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CONSTRAINT_COMMITHOOK,
        code: SQLITE_CONSTRAINT_COMMITHOOK,
        //str: "SQLITE_CONSTRAINT_COMMITHOOK".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CONSTRAINT_FOREIGNKEY,
        code: SQLITE_CONSTRAINT_FOREIGNKEY,
        //str: "SQLITE_CONSTRAINT_FOREIGNKEY".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CONSTRAINT_FUNCTION,
        code: SQLITE_CONSTRAINT_FUNCTION,
        //str: "SQLITE_CONSTRAINT_FUNCTION".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CONSTRAINT_NOTNULL,
        code: SQLITE_CONSTRAINT_NOTNULL,
        //str: "SQLITE_CONSTRAINT_NOTNULL".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CONSTRAINT_PRIMARYKEY,
        code: SQLITE_CONSTRAINT_PRIMARYKEY,
        //str: "SQLITE_CONSTRAINT_PRIMARYKEY".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CONSTRAINT_TRIGGER,
        code: SQLITE_CONSTRAINT_TRIGGER,
        //str: "SQLITE_CONSTRAINT_TRIGGER".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CONSTRAINT_UNIQUE,
        code: SQLITE_CONSTRAINT_UNIQUE,
        //str: "SQLITE_CONSTRAINT_UNIQUE".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CONSTRAINT_VTAB,
        code: SQLITE_CONSTRAINT_VTAB,
        //str: "SQLITE_CONSTRAINT_VTAB".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_CONSTRAINT_ROWID,
        code: SQLITE_CONSTRAINT_ROWID,
        //str: "SQLITE_CONSTRAINT_ROWID".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_NOTICE_RECOVER_WAL,
        code: SQLITE_NOTICE_RECOVER_WAL,
        //str: "SQLITE_NOTICE_RECOVER_WAL".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_NOTICE_RECOVER_ROLLBACK,
        code: SQLITE_NOTICE_RECOVER_ROLLBACK,
        //str: "SQLITE_NOTICE_RECOVER_ROLLBACK".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_WARNING_AUTOINDEX,
        code: SQLITE_WARNING_AUTOINDEX,
        //str: "SQLITE_WARNING_AUTOINDEX".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_AUTH_USER,
        code: SQLITE_AUTH_USER,
        //str: "SQLITE_AUTH_USER".to_string(),
    },
    ExtendedRow {
        id: ExtendedRC::SQLITE_OK_LOAD_PERMANENTLY,
        code: SQLITE_OK_LOAD_PERMANENTLY,
        //str: "SQLITE_OK_LOAD_PERMANENTLY".to_string(),
    }
];


// Little endian == least significant bit first.
// Note: Least significant bit of an extended result code is always primary result code.
fn is_extended(code: &u32) -> bool {
    code.to_le_bytes()[1] > 0
}

fn get_primary(code: &u32) -> u32 {
    code.to_le_bytes()[0] as u32
}

// @todo/low Use a map instead of iteration.
fn get_primary_row(primary_code: &u32) -> PrimaryRow {
    match PRIMARYTBL.iter().find(|&row| &row.code == primary_code) {
        Some(row) => (*row).clone(),
        None => panic!("Could not find enum/str for SQLite primary error code {}. Note: Converting FFI error codes to enums should never fail.", primary_code)
    }
}

pub fn get_primary_row_by_enum(e: &PrimaryRC) -> PrimaryRow {
    match PRIMARYTBL.iter().find(|&row| &row.id == e) {
        Some(row) => (*row).clone(),
        None => panic!("Could not find primary row via enum {:?}. Note: Converting FFI error codes to enums should never fail.", e)
    }
}

fn get_extended_row(extended_code: &u32) -> ExtendedRow {
    match EXTENDENDEDTBL.iter().find(|&row| &row.code == extended_code) {
        Some(row) => (*row).clone(),
        None => panic!("Could not find enum/str for SQLite primary error code {}. Note: Converting FFI error codes to enums should never fail.", extended_code)
    }
}


// Note: Extended result codes are all errors.
// Assumption: Even with `sqlite3_extended_result_codes` on, result codes can just be primary.
pub fn get_rows(code: &u32) -> (PrimaryRow, Option<ExtendedRow>) {

    // Primary
    let primary_code = get_primary(&code);
    let primary_row = get_primary_row(&primary_code);

    // Extended
    let mut extended_row = None;
    if is_extended(&code) {
        extended_row = Some(get_extended_row(&code));
    }

    (primary_row, extended_row)
}