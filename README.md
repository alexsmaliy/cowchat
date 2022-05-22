Cowchat
=======

This is a minimal [actix-web](https://docs.rs/actix-web/latest/actix_web/) server that exposes some REST endpoints and some basic WebSocket functionality. The persistence layer is SQLite.

Install SQLite3, run the [initdb.sh](./initdb.sh) script, then build or run using Cargo. The server listens on `localhost:3000`.