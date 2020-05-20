# Using Dtrace


- `sudo dtrace -s b.d -c "cargo test test_multiple_concurrent_runtimes  --package runtime -- --nocapture --test-threads=1" -l`
    - `-l` will list probes
        - `cargo x` does not work because it can not find workspace, even though the probes are listed?
    - `sudo` required on macOS because of SIP.

