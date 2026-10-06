# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
While the major version is 0, compatible additions bump the patch; breaking
changes to the generated code or the runtime bump the minor.

## [Unreleased]

The project was renamed from Contratto before its first release: runtime
crate `contract`, plugin `protoc-gen-contract-rust`, generated files
`<stem>.__contract.rs`.

### Added

- `protoc-gen-contract-rust` emits, per protobuf `service`:
  - `<Service>Sync`, whose methods return `Result<Reply, contract::Status>`.
    It is dyn-compatible: `Arc<dyn <Service>Sync>`.
  - `<Service>Async`, whose methods return
    `impl Future<Output = Result<Reply, Status>> + Send`, so implementations
    write plain `async fn` and generic callers pay no allocation.
  - `Dyn<Service>Async`, a cloneable handle that holds any `<Service>Async`
    implementation behind dynamic dispatch and implements the trait itself.

  Requests are taken by value and replies returned owned. Comments in the
  `.proto` become rustdoc.
- Streaming methods of every kind: server, client, and bidirectional. The
  sync form takes and returns `contract::BoxIter<'static, _>`, the async
  form `impl contract::Stream`; items are `Result<_, Status>`, and a reply
  stream sits inside the call's `Result`. No stream borrows the service.
- Bridges between the forms, for every kind of method: `contract::Inline`
  (a sync implementation's async form, run inside `poll`),
  `contract::Offload` (the same on tokio's blocking pool), and
  `contract::Blocking` (an async implementation's sync form). `Offload`
  and `Blocking` come with the runtime's `tokio` feature; the plugin
  parameter `gate_tokio_feature[=<name>]` puts their impls behind a Cargo
  feature. `Blocking` inside a current-thread runtime fails with
  `FAILED_PRECONDITION` instead of panicking. The bridges' helpers are
  associated functions, so an RPC named `Call` or `Feed` stays reachable
  with method syntax, and every bridge ends a stream after its first `Err`.
- `Dyn<Service>Async` implements `Debug`.
- README: the three generated items with the `Dyn<Service>Async`
  handle's API, an implementation, how to depend on the runtime (its
  `tokio` feature, or `gate_tokio_feature`), and the bridges' limits.
- Release binaries of the plugin for Linux, macOS, and Windows on amd64
  and arm64, static where the platform allows (macOS links only system
  libraries), with their sha256 in `checksums-sha256.txt`; a version tag
  builds, tests, and publishes them.
- Plugin parameters `buffa_module=`, `extern_path=`, `file_per_package`,
  and `element_memory_limit=`, as `protoc-gen-connect-rust` reads them, plus
  `runtime=` for the path of the runtime crate and `gate_tokio_feature`.
  `buffa_module=crate` maps the buffa output to the crate root. Message paths are absolute
  and match `protoc-gen-buffa` 0.9.2. Output is a `<stem>.__contract.rs`
  per proto and a `<pkg>.mod.rs` stitcher per package for
  `protoc-gen-buffa-packaging` (`filter=services`), or one
  `<dotted.pkg>.rs` per package with `file_per_package`.
- The plugin refuses, with a message that names the proto element: a
  message type no mapping covers, two methods with one Rust name, a
  generated name that another item of the package's module (buffa,
  connect-rust, or Contract) already has, and any unknown parameter.
- Runtime crate `contract`: `Status` (code, message, `Any` details; a
  mirror of `google.rpc.Status`), `Code` (the 17 `google.rpc.Code` values),
  `BoxFuture`, `BoxIter`, `BoxStream`, `Stream` (re-exported from
  `futures-core`), `IterStream`, and with `tokio` `ChannelStream`.

## Links

- [Unreleased]

[Unreleased]: https://github.com/sonalect/protoc-gen-contract-rust/commits/main
