# Contract

Plain Rust traits for protobuf services.

`protoc-gen-contract-rust` turns each `service` into plain Rust traits
over the message structs [buffa](https://github.com/anthropics/buffa)
generates. A component in the same process is called by a method call: no
encoding, no transport, no view types. The runtime crate `contract` holds
the types the generated code names.

```protobuf
service GreeterService {
  // Greet a person by name.
  rpc Greet(GreetRequest) returns (GreetReply);
}
```

```rust
pub trait GreeterServiceSync: Send + Sync {
    /// Greet a person by name.
    fn greet(&self, request: GreetRequest) -> Result<GreetReply, contract::Status>;
}

pub trait GreeterServiceAsync: Send + Sync {
    /// Greet a person by name.
    fn greet(&self, request: GreetRequest)
        -> impl Future<Output = Result<GreetReply, contract::Status>> + Send;
}

/// Any `GreeterServiceAsync` behind dynamic dispatch; implements it too.
#[derive(Clone)]
pub struct DynGreeterServiceAsync { /* … */ }
```

Implement the form natural to the work: sync for computation, async (with
plain `async fn`) for work that waits on I/O. Hold an implementation chosen
at run time as `Arc<dyn GreeterServiceSync>` or `DynGreeterServiceAsync`;
generic callers (`impl GreeterServiceAsync`) pay no allocation.

Streaming methods of every kind are generated. The sync form uses
`contract::BoxIter`, the async form `contract::Stream`; stream items are
`Result<_, Status>`:

```protobuf
service FeedService {
  rpc Watch(WatchRequest) returns (stream Item);  // server streaming
  rpc Collect(stream Item) returns (Summary);     // client streaming
  rpc Echo(stream Item) returns (stream Item);    // bidirectional
}
```

```rust
pub trait FeedServiceSync: Send + Sync {
    fn watch(&self, request: WatchRequest)
        -> Result<BoxIter<'static, Result<Item, Status>>, Status>;
    fn collect(&self, requests: BoxIter<'static, Result<Item, Status>>) -> Result<Summary, Status>;
    fn echo(&self, requests: BoxIter<'static, Result<Item, Status>>)
        -> Result<BoxIter<'static, Result<Item, Status>>, Status>;
}

pub trait FeedServiceAsync: Send + Sync {
    fn watch(&self, request: WatchRequest) -> impl Future<
        Output = Result<impl Stream<Item = Result<Item, Status>> + Send + use<Self>, Status>,
    > + Send;
    fn collect<R>(&self, requests: R) -> impl Future<Output = Result<Summary, Status>> + Send
    where
        R: Stream<Item = Result<Item, Status>> + Send + 'static;
    fn echo<R>(&self, requests: R) -> impl Future<
        Output = Result<impl Stream<Item = Result<Item, Status>> + Send + use<Self, R>, Status>,
    > + Send
    where
        R: Stream<Item = Result<Item, Status>> + Send + 'static;
}
```

An `Err` before the first item fails the call; an `Err` item ends the
stream. No stream borrows the service.

## Bridges

Each form is also implemented for wrappers around the other, so a caller
of one form can use an implementation of the other:

| Wrapper | Gives | How |
| - | - | - |
| `contract::Inline<T>` | a sync impl the async form | runs the call inside `poll`; for calls of microseconds |
| `contract::Offload<T>` | a sync impl the async form | runs the call on tokio's blocking pool (about 20 µs a call) |
| `contract::Blocking<T>` | an async impl the sync form | blocks the caller on a tokio runtime handle |

`Offload` and `Blocking` need the runtime crate's `tokio` feature; the
plugin parameter `gate_tokio_feature` puts their impls behind a feature of
the consuming crate.

```rust
let service = DynGreeterServiceAsync::new(Offload::new(MySyncGreeter, handle.clone()));
let sync = Blocking::new(service.clone(), handle);
```

## Generate

Run the plugin next to `protoc-gen-buffa`, each into its own `out`
directory:

```yaml
version: v2
plugins:
  - local: protoc-gen-buffa
    out: src/generated/buffa
  - local: protoc-gen-buffa-packaging
    out: src/generated/buffa
    strategy: all
  - local: protoc-gen-contract-rust
    out: src/generated/contract
    opt: [buffa_module=crate::proto]
  - local: protoc-gen-buffa-packaging
    out: src/generated/contract
    strategy: all
    opt: [filter=services]
```

```rust
#[path = "generated/buffa/mod.rs"]
pub mod proto;
#[path = "generated/contract/mod.rs"]
pub mod traits;
```

The crate depends on `contract` and on what buffa's output needs
(`buffa`, and `buffa-types` for well-known types).

| Parameter | Meaning |
| - | - |
| `buffa_module=<path>` | where the buffa output is mounted; shorthand for `extern_path=.=<path>` |
| `extern_path=<proto>=<path>` | maps a proto package prefix to a Rust module; repeatable, longest prefix wins |
| `file_per_package` | one `<dotted.pkg>.rs` per package instead of per-proto files and a stitcher |
| `element_memory_limit=<bytes\|unlimited>` | decode bound of the request, as for `protoc-gen-buffa` |
| `runtime=<path>` | path of the runtime crate; default `::contract` |
| `gate_tokio_feature[=<name>]` | `Offload` and `Blocking` impls under `#[cfg(feature = "<name>")]`; default `tokio` |

Paths are absolute (`::…`, `crate`, or `crate::…`). Any other parameter
fails the run.

## Status

Under development; the design and its stages are in [DESIGN.md](DESIGN.md).
Connect adapters and validation come next.

## Build

```bash
cargo test --workspace
bazel test //...            # the gate: golden files, clippy, lint, markdown
bazel run //proto:generate  # refresh the golden files after a plugin change

# a static release binary of the plugin, as the release workflow builds it
cargo build --locked --profile dist --target x86_64-unknown-linux-musl -p protoc-gen-contract-rust
```

A version tag `vX.Y.Z` runs the release workflow: Bazel build and tests on
Linux, macOS, and Windows, each on amd64 and arm64, then a GitHub Release
with the static plugin binary for each platform and their `SHA256SUMS`.

## License

Apache-2.0.
