# Pokedex For Fun

That is a simple _Pokedex_ REAST API server that expose

- `/HTTP/GET /pokemon/<pokemon name>` : Given a Pokemon name return a Pokemon info
- `HTTP/GET /pokemon/translated/<pokemon name>` : Given a Pokemon name return a Pokemon info where the description is
 translated in  `Yoda` or `Shakespeare` language. It uses Yoda for legendary pokemon or for the one that come from `cave`
 habitat.

## TL;DR

Build docker image:

```shell
docker build . -t tl-ex
```

And run the server to expose the service on `localhost:5000`

```shell
docker run -p 5000:5000 tl-ex
```

Query it

```shell
curl -sS 'http://localhost:5000/pokemon/translated/mewtwo' | jq .
```

```shell
curl -sS 'http://localhost:5000/pokemon/pikachu' | jq .
```

## **ATTENTION**

Some unit tests can fail if you run the test suite more than once in less than 60 seconds. This is 
due to the `funtranslations.mercxry.me` rate limiter that cannot accept more than 5 request per minute.

There are 2 unit tests on translation and 3 acceptance tests on `pokemon/translated` API: The next run
in the same minute will fail. I know that brittle tests are really annoying, but I should test the provider 
implementation and mock the concrete provider doesn't have too much sense.

## Compile

In order to compile the project you need standard [Rust toolchain installed](https://rust-lang.org/tools/install/). 

### Build

```shell
cargo build --release
```
Exec binary is in `./target/release/tl-ex`, to run it directly from cargo:

```shell
cargo run --release
```

### Test

```shell
cargo test
```

### Cargo Make

Optionally, if you want to run all CI steps you can install `cargo-make` 

```shell
cargo install cargo-make
```
and run 

```shell
cargo make ci
```

#### Coverage

```shell
cargo make cov-report
```
Save a `lcov.info`, `coverage_report.json`, and dump a report on terminal.

## Structure

```text
├── Cargo.lock
├── Cargo.toml
├── Dockerfile
├── Makefile.toml                            # cargo-make configuration
├── README.md                                # This file
├── src
│   ├── funtranslation_provider.rs           # Translator provider
│   ├── language_policies.rs                 # Implementation Selector language
│   ├── main.rs                              # Server start and routing logic
│   ├── rustemon_provider.rs                 # `rustemon` crate wrapper
│   ├── service.rs                           # Internal service and traits
│   ├── tests
│   │   └── utils.rs                         # Mocks and Fakes for testing
│   └── tests.rs                             # integration test with mocks and fakes
└── tests
    └── acceptance.rs                        # Acceptance test: run the server and test it e2e
```

## What is missed in this project

### Logging VS Tracing

We decided to use just `log` here and don't log too much stuff. A production ready project should use
tracing instead that can leverage on `span` and make async code simpler to follow in the logs. Tracing
can be also used to collect metrics on remote server and analyze them.

### Error handling

Current project use a lazy and minimal error handling. Use `thiserror` crate, but the domain errors
have a very poor granularity (`NotFound` and `Unknown`). A production ready project should provide 
a better Error taxonomic and translate better the providers error in order to handle better error 
situations.

Anyway, my advice is to not make the `reqwest`'s errors transparent and blow up them to our core logic.

### Pokemon Provider Configuration

We decided to not expose the `rustemon` configurations to the server implementation and just stick
on the default one. A production ready project should provide a set of configurations and reflect them in
the useful `rustemon` ones.

### Translator Provider

The translator provider implementation is very naive and plain, error handling is just minimal. Moreover, 
we didn't provide any cache; a fast way to provide it can be to leverage on the one provided by `reqwest`.

In a project like this with just two state-less API maybe a cache implemented on the access proxy service 
can be the best solution.

### Https 

In this case the best pattern is to leave the server with just `http` and implement the TLS in the 
access proxy like `nginx`.

### Docker

We decided to not leverage on `cargo-chef` and its docker image for this project. It can help a lot 
on reducing deploy time, but we don't know where it will be deployed and if we'll really need it.

### CI

We didn't provide any github action for this project. Anyway it can be a little wrapper that use 
`cargo make ci-remote`.
