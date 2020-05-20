https://github.com/mattn/go-sqlite3
- Add similar connection options


SQLite API
- `current_lock()`
    - Returns the current lock being held by a transaction.
    - https://www.sqlite.org/lockingv3.html#shared_lock
    
- `get_read_lock()`
    - Efficent way to get a read lock without doing a `SELECT * FROM sqlite_master`
        - This results in "SQLITE_SCHEMA" error if another connection changes the schema.
        
- `SQLITE_DBCONFIG_DEFENSIVE`
    - Connection option to prevent commands that can corrupt the database.

- `SQLITE_PREPARE_PERSISTENT`
    - Cache prepared statements.

- `SQLITE_ENABLE_SETLK_TIMEOUT` compile flag.
    - Use the OS to determine when a lock is released, instead of waiting for a specific amount of time.
    
- Make read/write open mode match Rusts ownership semantics.
    - E.g. When you have a read transaction, the underlying db connection is read only.
    - `https://www.sqlite.org/c3ref/open.html`
