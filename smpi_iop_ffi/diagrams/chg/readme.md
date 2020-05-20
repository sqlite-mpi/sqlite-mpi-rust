This shows a rough overview of the interactions between the Client, Host and Guest.

### Terms
- ST = Stream
- RR = Request Response.
- Host = the process that embeds the FFI.
- Guest = the FFI code, including background threads.
- Client = the language API and process.
    
### Explanation

- The client is possibly remote from the host, and only can send/receive messages to the host.
    - The client to host interface can be one of:
        - A: (i: ST, o: ST)
        - B: (i: RR, o: ST).
    -  The reason for the client generating the input message ID instead of the guest is to:
        - 1. Allow A to be implemented (without expecting a RR sync return from input messages - like in ST-based WebSockets). 
        - 2. Reduce the amount of state transitions in B to simplify clients.
            - E.g. Its impossible for the B.i response to arrive after the B.o event to the same request.
                - This can occur when:
                    - The guest generates the ID, and
                    - The host buffers the messages making the sequence of messages arriving on the client non-deterministic. 
                - This is an issue because
                     - The client is expected to keep a hashmap of `input_msg_id => Promise`
                     - It makes logic more complicated if the (in_msg_id, out_msg) response can arrive at the client before the input_msg_id.
                    

Question: Why not use a single RR interface instead of A or B?

- 1. The C FFI does not support promises, only A.
    - Input function will return immediately, offloading task to bg thread.
    - Output callback will be called with the result.

- 2. To support output events (without an input request).
    - Error logs.
    - Watching queries for changes. 



### Notes
- Diagrams created with https://sequencediagram.org.
