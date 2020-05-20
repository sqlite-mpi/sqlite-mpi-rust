# SMPI - SQLite Message Passing Interface

- This is a Rust based project that allows using SQLite from processes that cannot embed a C FFI.
    - E.g. a JS VM that can only send and receive messages using HTTP `fetch`.

- It wraps SQLites FFI and exports its own FFI.
    - Instead of the 100's of functions in SQLite, it exports a few functions that allow passing JSON messages that represent SQL queries.
    - Because JSON is serializable, these messages can pass over process and network boundaries.
        - This is not true of the SQLite FFI as it uses C pointers which ties usage to a single OS process.
    - Rust is used to cross compile a single program to many architectures.
        - This allows the same behavior over many different platforms.

- Its designed to be used in React Native mobile apps.
    - RN apps have a single JS code base that interacts with two host runtimes (iOS, Android) via message passing.
    - This Rust project has no specific RN code and is general enough to be used in other projects.

See https://sqlitempi.com/ for details.

# Rust workspace/packages:
Rust workspaces are used to separate layers:

- `sqliteffi`
    - Converts SQLites FFI into a Rust API.
- `sma`
    - "State Machine A"
    - Provides a more restricted API to `sqliteffi`.
        - E.g.
            - WAL mode is forced.
            - Ownership is used to prevent multiple conflicting transactions starting on the same DB connection handle.
            - Reads are concurrent, writes queue.
    - Its still completely synchronous - no threading, promises or runtime.
- `runtime`
    - Uses the `sma` and provides a thread based runtime.
    - Message passing is asynchronous.
        - After the message is passed to the background thread the function returns to the caller.
        - Once the message is processed by the background thread and has a reply, a callback notifies the original caller.
- `smpi_iop_ffi`
    - Uses `runtime` to wrap the API in various FFI standards.
    - Compiles binaries that export FFI for various platforms:
        - Android
        - iOS
        - Node.js
    - See `smpi_iop_ffi/sh/*`
    
    
Each package has tests; run `cargo test` from the package directory.

### Contact

Contact me for any reason emadda.dev@gmail.com.

Copyright © 2019 Enzo <emadda.dev@gmail.com>