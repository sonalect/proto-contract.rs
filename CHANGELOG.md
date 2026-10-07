# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
While the major version is 0, compatible additions bump the patch; breaking
changes to the generated code or the runtime bump the minor.

## [Unreleased]

### Added

- `examples/glossary`: a contract in use. A proto with a service and its
  error codes, the crate generated from it, a provider that implements the
  traits, a consumer that calls them without knowing the provider, and the
  application that wires the two (`cargo run -p glossary-app`).
- Open error codes: `protocontract::ErrorCode`, implemented by every protobuf
  enum that `protoc-gen-buffa` generates and by hand for a Rust enum, and
  `protocontract::Code`, a code with its type erased. A contract declares the
  codes its callers may branch on as an `enum` next to its `service`.
- `protocontract::Fault`, the trait of a failure (`code`, `retryable`,
  `backtrace`), and `protocontract::Failure`, the ready-made fault (code,
  message, `with_retryable`, `with_source`, a captured backtrace). Any
  `Fault` converts into `protocontract::Error` with `?`.
- `Error::is`, `code_as`, `retryable`, `stack`, `backtrace`,
  `downcast_ref`, and `context`; the `protocontract::Context` extension adds a
  stack frame to the error of a `Result` (`.context("load")`,
  `.with_context(|| …)`).
- The plugin finds a service's error-code enum by name
  (`GreeterErrorCode`, then `GreeterServiceErrorCode`, in the package of
  `GreeterService`), emits the alias `GreeterServiceErrorCode` next to the
  traits, and names it in their rustdoc. A service without such an enum is
  generated as before; a message or enum already named like the alias
  fails the run.
- `protocontract::RuntimeCode`: `PANICKED` (`Offload`), `CANNOT_BLOCK`
  (`Blocking` inside a current-thread runtime), `CANCELLED` (a blocking
  task dropped by its runtime).

### Changed

- **Breaking.** The runtime crate `contract` is now `protocontract`
  (directory `rust/protocontract`, Bazel target `//rust/protocontract`).
  Depend on `protocontract` and replace `contract::` paths; the plugin's
  `runtime=` default is `::protocontract`. A crate that mounts the traits
  may now name their module `contract`.
- **Breaking.** `contract::Status` is now `protocontract::Error`: every
  generated method returns `Result<_, protocontract::Error>`. Build one with
  `Error::new(code, message)`, where `code` is a value of the contract's
  error enum, or from a `Failure` or a `Fault` of your own.
- **Breaking.** The bridges report `RuntimeCode` values instead of
  `INTERNAL` and `FAILED_PRECONDITION`.
- The runtime depends on `buffa` (for `ErrorCode` on its enums) instead of
  `buffa-types`.
- Each method's parameter is named after its message in snake case
  (`word: Word`, `greet_request: GreetRequest`), plural for an inbound
  stream (`items`), instead of `request` and `requests`. Implementations
  are unaffected: Rust does not bind parameter names.

### Removed

- **Breaking.** The fixed `google.rpc.Code` set (`contract::Code` as an
  enum) and the per-code constructors (`Status::invalid_argument`, …):
  codes are open (see Added).
- **Breaking.** `Status::with_details`, `details`, and `into_parts`: the
  calls stay in one process, so the wire payloads of `google.rpc.Status`
  have no use.
- **Breaking.** `PartialEq` on the error: a fault cannot be compared.
  Compare codes (`error.is(code)`) instead.

## [0.1.0] - 2026-10-06

First release.

### Added

- `protoc-gen-contract-rust`, a protoc plugin that emits, per protobuf
  `service`:
  - `<Service>Sync`, whose methods return `Result<Reply, contract::Status>`.
    It is dyn-compatible: `Arc<dyn <Service>Sync>`.
  - `<Service>Async`, whose methods return
    `impl Future<Output = Result<Reply, Status>> + Send`, so implementations
    write plain `async fn` and generic callers pay no allocation.
  - `Dyn<Service>Async`, a cloneable handle (`new`, `from_arc`) that holds
    any `<Service>Async` implementation behind dynamic dispatch, implements
    the trait itself, and implements `Debug`.

  Requests are taken by value and replies returned owned. Comments in the
  `.proto` become rustdoc.
- Streaming methods of every kind: server, client, and bidirectional. The
  sync form takes and returns `contract::BoxIter<'static, _>`, the async
  form `impl contract::Stream`; items are `Result<_, Status>`, a reply
  stream sits inside the call's `Result`, and no stream borrows the
  service.
- Bridges between the forms, for every kind of method: `contract::Inline`
  (a sync implementation's async form, run inside `poll`),
  `contract::Offload` (the same on tokio's blocking pool), and
  `contract::Blocking` (an async implementation's sync form). `Offload`
  and `Blocking` come with the runtime's `tokio` feature. `Blocking` inside
  a current-thread runtime fails with `FAILED_PRECONDITION` instead of
  panicking, and every bridge ends a stream after its first `Err`.
- Plugin parameters `buffa_module=` (`crate` included), `extern_path=`,
  `file_per_package`, and `element_memory_limit=`, as
  `protoc-gen-connect-rust` reads them, plus `runtime=` for the path of the
  runtime crate and `gate_tokio_feature[=<name>]` to put the `Offload` and
  `Blocking` impls behind a Cargo feature. Message paths are absolute and
  match `protoc-gen-buffa` 0.9.2. Output is a `<stem>.__contract.rs` per
  proto and a `<pkg>.mod.rs` stitcher per package for
  `protoc-gen-buffa-packaging` (`filter=services`), or one
  `<dotted.pkg>.rs` per package with `file_per_package`. The output can sit
  next to `protoc-gen-connect-rust`'s, in the same modules.
- The plugin refuses, with a message that names the proto element: a
  message type no mapping covers, two methods with one Rust name, a
  generated name that another item of the package's module (buffa,
  connect-rust, or Contract) already has, and any unknown parameter.
- Runtime crate `contract`: `Status` (code, message, `Any` details; a
  mirror of `google.rpc.Status`), `Code` (the 17 `google.rpc.Code` values),
  `BoxFuture`, `BoxIter`, `BoxStream`, `Stream` (re-exported from
  `futures-core`), `IterStream`, `Inline`, and with `tokio` `ChannelStream`,
  `Offload`, and `Blocking`.
- Release binaries of the plugin for Linux, macOS, and Windows on amd64
  and arm64, static where the platform allows (macOS links only system
  libraries), with their sha256 in `checksums-sha256.txt`.

## Links

- [Unreleased]
- [0.1.0]

[Unreleased]: https://github.com/sonalect/proto-contract.rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/sonalect/proto-contract.rs/releases/tag/v0.1.0
