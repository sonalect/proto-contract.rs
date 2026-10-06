//! Runtime for the code that `protoc-gen-contratto-rust` generates.
//!
//! For each protobuf `service`, the plugin emits:
//!
//! - `<Service>Sync`: each method returns `Result<Reply, Status>` when the
//!   call is done. Streams are [`BoxIter`]s. The trait is dyn-compatible,
//!   so an implementation chosen at run time is held as
//!   `Arc<dyn GreeterServiceSync>`.
//! - `<Service>Async`: each method returns `impl Future` of the same
//!   result, so an implementation writes plain `async fn`. Streams are
//!   [`Stream`]s. Generic callers (`impl GreeterServiceAsync`) pay no
//!   allocation.
//! - `Dyn<Service>Async`: a cloneable handle that holds any `<Service>Async`
//!   implementation behind dynamic dispatch and implements the trait
//!   itself. Each call through it boxes the future ([`BoxFuture`]) and the
//!   reply stream ([`BoxStream`]).
//!
//! Every method takes its request by value and returns its reply owned; a
//! call in the same process is a plain method call, with no encoding and
//! no transport.
//!
//! Streaming methods, per form:
//!
//! | Method | `<Service>Sync` | `<Service>Async` |
//! | - | - | - |
//! | server streaming | `Result<BoxIter<Result<Reply, Status>>, Status>` | `Result<impl Stream<Item = Result<Reply, Status>>, Status>` |
//! | client streaming | takes `BoxIter<Result<Request, Status>>` | takes `impl Stream<Item = Result<Request, Status>>` |
//! | bidirectional | both | both |
//!
//! An `Err` before the first item fails the whole call; an `Err` item ends
//! a stream with that failure. Inbound items are `Result`s because a stream
//! that crosses a process boundary can break, and the implementation must
//! see that rather than a stream that merely ends early.

mod code;
mod status;

use std::future::Future;
use std::pin::Pin;

pub use code::Code;
pub use futures_core::Stream;
pub use status::Status;

/// The future a call through a `Dyn<Service>Async` handle returns: boxed,
/// `Send`, and borrowing for at most `'a`.
///
/// ```
/// use contratto::{BoxFuture, Status};
///
/// fn double(value: i64) -> BoxFuture<'static, Result<i64, Status>> {
///     Box::pin(async move { Ok(value * 2) })
/// }
/// # let _ = double(2);
/// ```
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A stream of a sync streaming method: a boxed `Send` iterator that
/// borrows for at most `'a`.
///
/// ```
/// use contratto::{BoxIter, Status};
///
/// let replies: BoxIter<'static, Result<u32, Status>> = Box::new((1..=3).map(Ok));
/// assert_eq!(replies.count(), 3);
/// ```
pub type BoxIter<'a, T> = Box<dyn Iterator<Item = T> + Send + 'a>;

/// A stream of an async streaming method called through a
/// `Dyn<Service>Async` handle: boxed, pinned, and `Send`.
pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + 'a>>;
