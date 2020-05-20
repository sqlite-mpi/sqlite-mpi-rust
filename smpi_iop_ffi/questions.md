

#### Question: Why allow returning both sync and async from the input FFI function?
Issue: With only async replies from the FFI (client receives `in_msg_id`, waits for callback), its not possible to return a sync response.
- E.g. In JS you can optionally wrap a sync variable in a promise:
     - `const f = async () => (x ? Promise.resolve("sync var") : await realAsyncFn())`
- With the FFI, it would be useful to return sync replies for these classes of error:
     - Input msg parse error, background thread channel issues.
         - These errors:
             - Occur before entering the SMPI event loop.
                 - The SMPI event loop should not know or handle these errors - its a different layer.
             - Will differ depending on the encode/decode format being used (JSON, FlatBuffers etc).
             - Are computed in sync with the FFI host caller.
                 - The response is already known, so why convert it to an async response?
                     - Issues with converting a sync to async response:
                         - The `input_json` function has not returned yet, the output callback cannot be called until input_json returns.
                             - The host does not have the request id in its request map until `input_json` returns.

Fix: Allow the FFI to return sync or async (same semantics as the JS example above).
- This would allow changing the SMPI return values from async to sync in the future without changing the current integrations.
- Returning JSON allows for greater response flexibility by adding keys or returning different types.

Alternative fixes:
- C error handling (error pointer, overload string).
     - Cannot return sync fulfilled promises (non-errors).