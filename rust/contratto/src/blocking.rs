//! `Blocking`: the sync form of an async implementation.

use std::future::Future;
use std::pin::Pin;

use futures_core::Stream;
use tokio::runtime::{Handle, RuntimeFlavor};
use tokio::sync::mpsc;

use crate::offload::{STREAM_BUFFER, pump};
use crate::stream::next;
use crate::{BoxIter, ChannelStream, Status};

/// The sync form of an async implementation: each call blocks the calling
/// thread until the async method, run on the runtime behind the stored
/// handle, is done.
///
/// A call may come from any thread outside a runtime, from a
/// `spawn_blocking` closure, or from inside a multi-thread runtime, where
/// it moves the worker's other tasks away first (`block_in_place`) and
/// suspends any other future of the same task meanwhile. From inside a
/// current-thread runtime a call fails with `FAILED_PRECONDITION` instead:
/// blocking that runtime's only thread would stop the call it waits for.
/// The check cannot tell that runtime's `spawn_blocking` threads apart, so
/// they fail the same way; call from a `std::thread` there.
///
/// The handle should belong to a multi-thread runtime, or to a
/// current-thread runtime that another thread drives: a current-thread
/// runtime does not run its I/O and timers for a call made from elsewhere.
///
/// - A reply stream is an iterator whose `next` blocks for the next item;
///   each `next` is checked like a call.
/// - An inbound iterator is pulled on the runtime's blocking pool and handed
///   to the async method as a stream that holds up to 16 items ahead; a
///   panic of that iterator ends the stream with `Status::internal`.
///
/// A panic of the async method itself reaches the caller as a panic, as it
/// would from a direct call.
///
/// The plugin implements each `<Service>Sync` for `Blocking<T>` where `T`
/// implements `<Service>Async`.
#[derive(Clone, Debug)]
pub struct Blocking<T> {
    service: T,
    handle: Handle,
}

impl<T> Blocking<T> {
    /// The sync form of `service`, run on the runtime behind `handle`.
    pub fn new(service: T, handle: Handle) -> Blocking<T> {
        Blocking { service, handle }
    }

    /// The async implementation.
    pub fn get_ref(&self) -> &T {
        &self.service
    }

    /// The async implementation, moved out.
    pub fn into_inner(self) -> T {
        self.service
    }

    /// Block until `call` is done and return its result.
    ///
    /// # Errors
    ///
    /// `FAILED_PRECONDITION` when called inside a current-thread runtime;
    /// otherwise whatever `call` returns.
    pub fn block_on<R>(&self, call: impl Future<Output = Result<R, Status>>) -> Result<R, Status> {
        block_on(&self.handle, call)
    }

    /// Block until `call` returns its reply stream, and return the stream
    /// as an iterator that blocks for each item.
    ///
    /// # Errors
    ///
    /// As [`Blocking::block_on`].
    pub fn block_on_stream<X, S>(
        &self,
        call: impl Future<Output = Result<S, Status>>,
    ) -> Result<BoxIter<'static, Result<X, Status>>, Status>
    where
        S: Stream<Item = Result<X, Status>> + Send + 'static,
        X: Send + 'static,
    {
        let stream = self.block_on(call)?;
        Ok(Box::new(BlockingIter {
            handle: self.handle.clone(),
            stream: Some(Box::pin(stream)),
        }))
    }

    /// Pull `requests` on the runtime's blocking pool and return them as a
    /// stream for an async method. Dropping the stream stops the pull.
    pub fn feed<X: Send + 'static>(
        &self,
        requests: BoxIter<'static, Result<X, Status>>,
    ) -> ChannelStream<Result<X, Status>> {
        let (sender, receiver) = mpsc::channel(STREAM_BUFFER);
        drop(self.handle.spawn_blocking(move || pump(requests, &sender)));
        ChannelStream::new(receiver)
    }
}

/// Block the current thread on `future`, run on `handle`'s runtime.
fn block_on<R>(
    handle: &Handle,
    future: impl Future<Output = Result<R, Status>>,
) -> Result<R, Status> {
    check_can_block()?;
    tokio::task::block_in_place(|| handle.block_on(future))
}

/// `FAILED_PRECONDITION` inside a current-thread runtime, where blocking
/// would stop the runtime the call needs.
fn check_can_block() -> Result<(), Status> {
    match Handle::try_current() {
        Ok(current) if current.runtime_flavor() == RuntimeFlavor::CurrentThread => {
            Err(Status::failed_precondition(
                "a sync call through Blocking cannot wait inside a current-thread tokio \
                 runtime; call it from a std::thread or use a multi-thread runtime",
            ))
        }
        _ => Ok(()),
    }
}

/// An async reply stream read as a blocking iterator.
struct BlockingIter<S> {
    handle: Handle,
    /// `None` once the stream has ended or failed.
    stream: Option<Pin<Box<S>>>,
}

impl<X, S: Stream<Item = Result<X, Status>>> Iterator for BlockingIter<S> {
    type Item = Result<X, Status>;

    fn next(&mut self) -> Option<Result<X, Status>> {
        let stream = self.stream.as_mut()?;
        // A panic of the async stream runs on this thread and reaches the
        // caller as it would from a direct call.
        let item = match block_on(&self.handle, async { Ok(next(stream).await) }) {
            Ok(item) => item,
            Err(status) => Some(Err(status)),
        };
        if !matches!(item, Some(Ok(_))) {
            self.stream = None;
        }
        item
    }
}
