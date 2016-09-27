# Rustcast

A simple Internet Radio Station.

## Design Decisions
- Programming Language: Rust
- Server Design Model -- Async IO based using [`mio` library](https://github.com/carllerche/mio)
- I spawn OS threads for each station and communicate to those threads from my server (event loop/`poll` in the code) using Rust channels. Each station thread is responsible for streaming a single mp3 to a set of UDP clients.
- I depend on the `mio` poll mechanism to handle multiple clients.
- I have used the Rust [standard library networking APIs](https://doc.rust-lang.org/std/net/) in both of the clients, ie, no dependence on `mio` in those two programs.

## Known Bugs / Missing Functionality
- The server currently doesn't send announces for change of songs. I could have added it using another dedicated channel which would create another `send_message` for the relevant `connection` (I just ran out of time by the time I noticed this was missing). However, the TCP client is capable of handling simultaneous input from server and user as I use threads there as well.
- The server currently doesn't have a CLI. It was not hard to do, I just noticed it too late as it was not obvious from the reference implementation. I could have spawned another thread to take user input (similar to what I did in the TCP client), have a channel to communicate back to the server (event loop) and take appropriate actions.
- I am unsure if I have handled timeouts well in the async server. `mio` seemed to be missing APIs for the same, apart from the timeout argument to the primary `poll` function. I, otherwise, have setup timeout in the TCP client.

## Acknowledgements
- I learned most of basics of async programming and how to structure my program with `mio` from [Creating A Multi-echo Server using Rust and mio](http://hermanradtke.com/2015/07/22/creating-a-multi-echo-server-using-rust-and-mio.html) & associated posts and source code written by Herman J. Radtke III.
