### IO providers.

An IO provider is code that implements:

1. Protocol
    - Read/writing bytes from a specific system.
    
2. Encoding
    - Encode/decode bytes to and from the runtimes native data types. 

3. Event loop interaction.
    - Scheduling computation of functions within an runtimes event loop.
    - Integrates with an event loops input and output event queue API.


"IO providers" have the following attributes:

- Dependable.
    - They are package level modules.
    - They can be depended on with the languages native package manager system (NPM, Cargo, etc).
    

- Interchangeable.
    - They expose the same public interface so can be exchanged with other IO providers.
        - E.g.
            - 1. Different protocols (HTTP, Websockets etc).
            - 2. Different encodings (JSON, ProtocolBuffers, Flatbuffers etc).
            - 3. Different event loops (Node, C#, Python, Chrome).    

- Nestable.
    - Or "chain-able".
    - IO providers can depend on other IO providers to tunnel/bridge/pass messages.
    - E.g. Messages may have to be passed over a few different frameworks (RN).



Because IO providers implement (1, 2, and 3) and have the above attributes they allow:

- SMPI clients and servers can focus on processing messages in their native runtime datatypes.
    - E.g. JS objects, Rust enums/types.
    - Clients and servers work the same with any IO providers.


### Request/response vs event emitting.

The SMPI FFI provides a bidirectional stream of input and output messages, but the client code currently only uses request/response.

The event emitting message passing feature would allow streaming events like logs from the FFI to the host so it can log items in its native framework.

### Questions

Where do you convert IO into a request/response interface? 
- A. As close to the client as possible.
- B. As close to the FFI as possible.

Where does the `in_msg_id` terminate?
- What happens when the `input` FFI cannot be run sync to get the `in_msg_id` return value?
