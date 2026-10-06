# Contract

Plain Rust traits for protobuf services.

`protoc-gen-contract-rust` turns each protobuf `service` into Rust traits
over the message structs [buffa](https://github.com/anthropics/buffa)
generates. A component in the same process is called by a method call: no
encoding, no transport, no view types. The runtime crate `contract` holds
the types the generated code names. Requires Rust 1.99.

## What is generated

```protobuf
service GreeterService {
  // Greet a person by name.
  rpc Greet(GreetRequest) returns (GreetReply);
}
```

For each service, three items:

```rust
/// The blocking form. Dyn-compatible: `Arc<dyn GreeterServiceSync>`.
pub trait GreeterServiceSync: Send + Sync {
    /// Greet a person by name.
    fn greet(&self, request: GreetRequest) -> Result<GreetReply, contract::Status>;
}

/// The async form. Implementations write plain `async fn`; a generic
/// caller (`impl GreeterServiceAsync`) pays no allocation.
pub trait GreeterServiceAsync: Send + Sync {
    /// Greet a person by name.
    fn greet(&self, request: GreetRequest)
        -> impl Future<Output = Result<GreetReply, contract::Status>> + Send;
}

/// Any `GreeterServiceAsync` behind dynamic dispatch, for an implementation
/// chosen at run time. It implements `GreeterServiceAsync` itself; each call
/// boxes its future. Cloning shares the implementation; it also
/// implements `Debug`.
#[derive(Clone)]
pub struct DynGreeterServiceAsync { /* … */ }

impl DynGreeterServiceAsync {
    pub fn new<T: GreeterServiceAsync + 'static>(service: T) -> Self;
    pub fn from_arc<T: GreeterServiceAsync + 'static>(service: Arc<T>) -> Self;
}
```

Every method takes its request by value and returns its reply owned; the
error is `contract::Status`, a mirror of `google.rpc.Status`. Comments in
the `.proto` become rustdoc.

Implement the form natural to the work: sync for computation, async for
work that waits on I/O.

```rust
struct Greeter;

impl GreeterServiceAsync for Greeter {
    async fn greet(&self, request: GreetRequest) -> Result<GreetReply, Status> {
        if request.name.is_empty() {
            return Err(Status::invalid_argument("name is empty"));
        }
        Ok(GreetReply { text: format!("Hello, {}!", request.name), ..Default::default() })
    }
}

let greeter = DynGreeterServiceAsync::new(Greeter);
let reply = greeter.greet(request).await?;
```

### Streaming

Methods of every kind are generated. The sync form takes and returns
`contract::BoxIter`, the async form `contract::Stream`; the `Dyn…Async`
handle boxes them as `contract::BoxStream`. Stream items are
`Result<_, Status>`.

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
stream. No stream borrows the service: an implementation clones what its
stream needs, and a stream can outlive the call and move between threads.

## Bridges

Each form is also implemented for wrappers around the other, so a caller
of one form can use an implementation of the other:

| Wrapper | Gives | How |
| - | - | - |
| `contract::Inline<T>` | a sync impl the async form | runs the call inside `poll`; for calls of microseconds |
| `contract::Offload<T>` | a sync impl the async form | runs the call on tokio's blocking pool, about 20 µs a call |
| `contract::Blocking<T>` | an async impl the sync form | blocks the caller on a stored tokio runtime handle |

```rust
// A sync implementation, called through the async form.
let service = DynGreeterServiceAsync::new(Offload::new(MySyncGreeter, handle.clone()));
// The same, back in the sync form.
let sync = Blocking::new(service.clone(), handle);
```

- Every wrapper covers every kind of method, and each fits in a
  `Dyn…Async` handle or an `Arc<dyn …Sync>`.
- `Inline` reads an inbound stream to its end before the sync call, so a
  bidirectional call through it answers only after the last request.
  `Offload` and `Blocking` answer each request as it arrives.
- `Blocking` works from any thread outside a runtime and inside a
  multi-thread runtime; inside a current-thread runtime it fails with
  `FAILED_PRECONDITION` rather than block that runtime's only thread.
- `Offload` turns a panic of the sync implementation into
  `Status::internal`.

## Use

Get the plugin from the
[Releases](https://github.com/sonalect/protoc-gen-contract-rust/releases)
(from v0.1.0 on): static binaries for Linux, macOS, and Windows on amd64
and arm64, with `SHA256SUMS`. Or build it:

```bash
cargo install --locked --git https://github.com/sonalect/protoc-gen-contract-rust protoc-gen-contract-rust
```

Run it next to `protoc-gen-buffa`, each into its own `out` directory:

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

Mount both trees. Name the traits' module anything but `contract`: next to
the crate `contract` the path `contract::…` would be ambiguous.

```rust
#[path = "generated/buffa/mod.rs"]
pub mod proto;
#[path = "generated/contract/mod.rs"]
pub mod traits;
```

The crate depends on the runtime with its `tokio` feature, since the
generated code implements the traits for `Offload` and `Blocking`, and on
what buffa's output needs (`buffa`, and `buffa-types` for well-known
types). The runtime is published by git tag:

```toml
[dependencies]
contract = { git = "https://github.com/sonalect/protoc-gen-contract-rust", tag = "v0.1.0", features = ["tokio"] }
```

To keep tokio optional, pass `gate_tokio_feature` to the plugin: the
`Offload` and `Blocking` impls then sit behind a `tokio` feature of your
crate, which turns on the runtime's:

```toml
[dependencies]
contract = { git = "https://github.com/sonalect/protoc-gen-contract-rust", tag = "v0.1.0" }

[features]
tokio = ["contract/tokio"]
```

`protoc-gen-connect-rust` may run on the same protos into the same crate,
even into the same modules: Contract takes no name and no file that
connect-rust takes, and the run fails if a generated name meets an
existing one, naming both proto elements.

### Parameters

| Parameter | Meaning |
| - | - |
| `buffa_module=<path>` | where the buffa output is mounted; shorthand for `extern_path=.=<path>` |
| `extern_path=<proto>=<path>` | maps a proto package prefix to a Rust module; repeatable, longest prefix wins |
| `file_per_package` | one `<dotted.pkg>.rs` per package instead of per-proto files and a stitcher; drop the packaging step and mount each file yourself |
| `element_memory_limit=<bytes\|unlimited>` | decode bound of the request, as for `protoc-gen-buffa` |
| `runtime=<path>` | path of the runtime crate; default `::contract` |
| `gate_tokio_feature[=<name>]` | `Offload` and `Blocking` impls under `#[cfg(feature = "<name>")]`; default `tokio` |

Paths are absolute (`::…`, `crate`, or `crate::…`). Well-known types map
to `::buffa_types` on their own. Any other parameter fails the run.

## Status

Under development; the design, its stages, and its decisions are in
[DESIGN.md](DESIGN.md). Connect adapters (serving and calling a service
over Connect through the same traits) and request validation come next.

## Build

```bash
cargo test --workspace
bazel test //...            # the gate: golden files, clippy, lint, markdown
bazel run //proto:generate  # refresh the golden files after a plugin change

# a static release binary of the plugin, as the release workflow builds it
# (RUSTFLAGS unset: it would replace the static flags of .cargo/config.toml)
env -u RUSTFLAGS cargo build --locked --profile dist --target x86_64-unknown-linux-musl -p protoc-gen-contract-rust
```

An annotated version tag `vX.Y.Z` on `main` runs the release workflow:
Bazel build and tests on Linux, macOS, and Windows, each on amd64 and
arm64, then a GitHub Release with the static plugin binary for each
platform and their `SHA256SUMS`; the tag's message is the release notes.

## License

Apache-2.0.
