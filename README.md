# ![RealWorld Example App](logo.png)

> ### [YOUR_FRAMEWORK] codebase containing real world examples (CRUD, auth, advanced patterns, etc) that adheres to the [RealWorld](https://github.com/gothinkster/realworld) spec and API.


### [Demo](https://demo.realworld.io/)&nbsp;&nbsp;&nbsp;&nbsp;[RealWorld](https://github.com/gothinkster/realworld)


This codebase was created to demonstrate a fully fledged fullstack application built with [Tide](https://docs.rs/tide/latest/tide/) including CRUD operations, authentication, routing, pagination, and more.

We've gone to great lengths to adhere to the [Tide](https://docs.rs/tide/latest/tide/) community styleguides & best practices.

For more information on how to this works with other frontends/backends, head over to the [RealWorld](https://github.com/gothinkster/realworld) repo.


### How it works
This backend leverages rusts' [tide](https://docs.rs/tide/latest/tide/) 
framework for implementing the web app, and sqlite database managed as a
single file.
Simple configuration is available through .env file.

### Getting started

Install [rust](https://www.rust-lang.org/en-US/install.html)
```sh
# download rustup and install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Download and install [sqlite](https://www.sqlite.org/download.html) 
```sh
# for Ubuntu:
apt-get install sqlite3
```
Configure backend by editing .env file

Build and run backend
```sh
cargo run
```
### Configuration
You can run app with fresh database by setting DROP_DATABASE=1 in .env file.
Database location and host and port are configurable as well.

### Testing
Note: registering and logging in take some seconds because password hashing is CPU
consuming.
#### Postman testing 
Configure HOST=127.0.0.1 or delete HOST from .env and set HTTP_PORT=3000
and execute
```sh
cargo run
```
#### Integration Test
These tests do not cover http requests parsing, instead they 
introduce business logic testing steps by artificially forming 
middle layer server requests thus simulating local loop without involving tide framework.
Execute
```sh
cargo test -- --nocapture
```

