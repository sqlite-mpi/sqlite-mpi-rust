See https://sqlitempi.com/ for details.

### Rust workspace/packages:
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