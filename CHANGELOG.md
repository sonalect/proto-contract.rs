# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
While the major version is 0, compatible additions bump the patch; breaking
changes to the generated code or the runtime bump the minor.

## [Unreleased]

### Added

- `protoc-gen-contratto-rust` emits, per protobuf `service`:
  - `<Service>Sync`, whose methods return `Result<Reply, contratto::Status>`.
    It is dyn-compatible: `Arc<dyn <Service>Sync>`.
  - `<Service>Async`, whose methods return
    `impl Future<Output = Result<Reply, Status>> + Send`, so implementations
    write plain `async fn` and generic callers pay no allocation.
  - `Dyn<Service>Async`, a cloneable handle that holds any `<Service>Async`
    implementation behind dynamic dispatch and implements the trait itself.

  Requests are taken by value and replies returned owned. Comments in the
  `.proto` become rustdoc.
- Streaming methods of every kind: server, client, and bidirectional. The
  sync form takes and returns `contratto::BoxIter`, the async form
  `impl contratto::Stream`; items are `Result<_, Status>`, and a reply
  stream sits inside the call's `Result`. Async reply streams do not borrow
  the service.
- Plugin parameters `buffa_module=`, `extern_path=`, `file_per_package`,
  and `element_memory_limit=`, as `protoc-gen-connect-rust` reads them, plus
  `runtime=` for the path of the runtime crate. Message paths are absolute
  and match `protoc-gen-buffa` 0.9.2. Output is a `<stem>.__contratto.rs`
  per proto and a `<pkg>.mod.rs` stitcher per package for
  `protoc-gen-buffa-packaging` (`filter=services`), or one
  `<dotted.pkg>.rs` per package with `file_per_package`.
- The plugin refuses, with a message that names the proto element: a
  message type no mapping covers, two methods with one Rust name, a
  generated name that another item of the package's module (buffa,
  connect-rust, or Contratto) already has, and any unknown parameter.
- Runtime crate `contratto`: `Status` (code, message, `Any` details; a
  mirror of `google.rpc.Status`), `Code` (the 17 `google.rpc.Code` values),
  `BoxFuture`, `BoxIter`, `BoxStream`, and `Stream` (re-exported from
  `futures-core`).

## Links

- [Unreleased]

[Unreleased]: https://github.com/sonalect/protoc-gen-contratto-rust/commits/main
