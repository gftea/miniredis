# miniredis

Purpose: try writing miniredis to compare the design thinking with tokio's mini-redis example.

In below notes,  `self` refer to my own design, `tokio` refer to tokio's mini-redis example.

## encode Frame to bytes

**self**: `Frame` provide `encode` interface to encode frame into bytes, so provide interfaces for bidirectional conversion between bytes and Frame.

**tokio**: this is done in `Connection` type.

## parse Frame to command

**self**: `Frame` type provide interface to get iterator for Array Frame. define a trait  `Prase` that command requires, then implement the trait for the Frame iterator.

*'tokio'*:: this is done by create new struct type `Parse`.

## write buffer of Connection

**self**: use `BytesMut` as write buffer for `TcpStream`, so we manage the write buffer directly, also enable the `Frame` encode function to use same buffer.

**tokio**: wrap the `TcpStream` in type `BufWritter`.

## Why we need a `Request` type but not a `Response` type

**self**: Thinking about the request-response model.

Server receive `Request`, before parsing a received frame, it does not know which command, so we define `Request` type to unify all commands and provide interface to parse a frame to a specific command type. 

Client send `Request`, but it knows which command explicitly, so it can directly construct a command and send it .

Clien receive `Response`, according what command is sent, it know what frame types it expects as `Response`

Server send `Response`, it knows what frame to encode response so it can directly use `Frame` to construct a response.




