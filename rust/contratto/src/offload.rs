//! `Offload`: the async form of a sync implementation, on tokio's blocking
//! pool.

use std::any::Any;
use std::future::Future;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use futures_core::Stream;
use tokio::runtime::Handle;
use tokio::sync::{mpsc, oneshot};

use crate::stream::next;
use crate::{BoxIter, ChannelStream, Status};

/// How many items a stream moved between threads holds ahead of its
/// reader.
pub(crate) const STREAM_BUFFER: usize = 16;

/// The async form of a sync implementation: each call runs the sync method
/// on tokio's blocking pool, so the async side never blocks.
///
/// - A unary call costs one hop to the blocking pool and back.
/// - A reply stream is pulled on the blocking pool and handed over through
///   a channel that holds up to 16 items; dropping the stream stops the
///   pull.
/// - An inbound stream reaches the sync method as an iterator whose `next`
///   waits on the blocking thread, so a bidirectional call can answer each
///   request as it arrives.
///
/// A panic in the sync method becomes `Status::internal`: the call fails,
/// or the stream ends with that item. The work starts when the future is
/// first polled; once started, dropping the future does not stop a call
/// that is already running on the blocking pool.
///
/// The plugin implements each `<Service>Async` for `Offload<T>` where `T`
/// implements `<Service>Sync`. The helpers below are associated functions
/// (`Offload::call(&offload, …)`), so they never hide a trait method of the
/// same name.
#[derive(Debug)]
pub struct Offload<T> {
    service: Arc<T>,
    handle: Handle,
}

impl<T> Clone for Offload<T> {
    fn clone(&self) -> Offload<T> {
        Offload {
            service: Arc::clone(&self.service),
            handle: self.handle.clone(),
        }
    }
}

impl<T> Offload<T> {
    /// The async form of `service`, run on the blocking pool of the runtime
    /// behind `handle`.
    pub fn new(service: T, handle: Handle) -> Offload<T> {
        Offload::from_arc(Arc::new(service), handle)
    }

    /// The async form of a shared `service`.
    pub fn from_arc(service: Arc<T>, handle: Handle) -> Offload<T> {
        Offload { service, handle }
    }

    /// The sync implementation.
    pub fn get_ref(this: &Offload<T>) -> &T {
        &this.service
    }
}

impl<T: Send + Sync + 'static> Offload<T> {
    /// Run `call` on the blocking pool and return its result.
    pub fn call<R, F>(
        this: &Offload<T>,
        call: F,
    ) -> impl Future<Output = Result<R, Status>> + Send + 'static
    where
        F: FnOnce(&T) -> Result<R, Status> + Send + 'static,
        R: Send + 'static,
    {
        let (service, handle) = (Arc::clone(&this.service), this.handle.clone());
        async move {
            let task = handle.spawn_blocking(move || {
                catch_unwind(AssertUnwindSafe(|| call(&service)))
                    .unwrap_or_else(|panic| Err(panicked(&panic)))
            });
            task.await.unwrap_or_else(|error| {
                Err(Status::internal(format!("blocking call failed: {error}")))
            })
        }
    }

    /// Run `call`, which returns a reply stream, on the blocking pool, and
    /// pull that stream there.
    pub fn server_streaming<X, F>(
        this: &Offload<T>,
        call: F,
    ) -> impl Future<Output = Result<ChannelStream<Result<X, Status>>, Status>> + Send + 'static
    where
        F: FnOnce(&T) -> Result<BoxIter<'static, Result<X, Status>>, Status> + Send + 'static,
        X: Send + 'static,
    {
        let (service, handle) = (Arc::clone(&this.service), this.handle.clone());
        streaming(handle, move || call(&service))
    }

    /// Run `call` on the blocking pool with `requests` as an iterator and
    /// return its result.
    pub fn client_streaming<S, X, R, F>(
        this: &Offload<T>,
        requests: S,
        call: F,
    ) -> impl Future<Output = Result<R, Status>> + Send + 'static
    where
        S: Stream<Item = Result<X, Status>> + Send + 'static,
        X: Send + 'static,
        F: FnOnce(&T, BoxIter<'static, Result<X, Status>>) -> Result<R, Status> + Send + 'static,
        R: Send + 'static,
    {
        let inbound = this.handle.clone();
        Offload::call(this, move |service| {
            call(service, Box::new(StreamIter::new(inbound, requests)))
        })
    }

    /// Run `call` on the blocking pool with `requests` as an iterator, and
    /// pull the reply stream it returns there.
    pub fn bidirectional<S, X, Y, F>(
        this: &Offload<T>,
        requests: S,
        call: F,
    ) -> impl Future<Output = Result<ChannelStream<Result<Y, Status>>, Status>> + Send + 'static
    where
        S: Stream<Item = Result<X, Status>> + Send + 'static,
        X: Send + 'static,
        Y: Send + 'static,
        F: FnOnce(
                &T,
                BoxIter<'static, Result<X, Status>>,
            ) -> Result<BoxIter<'static, Result<Y, Status>>, Status>
            + Send
            + 'static,
    {
        let (service, handle) = (Arc::clone(&this.service), this.handle.clone());
        let inbound = handle.clone();
        streaming(handle, move || {
            call(&service, Box::new(StreamIter::new(inbound, requests)))
        })
    }
}

/// Run `start` on the blocking pool; if it returns a stream, pull it there
/// into a channel and return the channel's reading end.
async fn streaming<X, F>(
    handle: Handle,
    start: F,
) -> Result<ChannelStream<Result<X, Status>>, Status>
where
    F: FnOnce() -> Result<BoxIter<'static, Result<X, Status>>, Status> + Send + 'static,
    X: Send + 'static,
{
    let (sender, receiver) = mpsc::channel(STREAM_BUFFER);
    let (started, start_result) = oneshot::channel();
    // The task runs until the stream ends or its reader goes away; its
    // outcome reaches the reader through the two channels.
    drop(handle.spawn_blocking(move || {
        let items = match catch_unwind(AssertUnwindSafe(start)) {
            Ok(Ok(items)) => items,
            Ok(Err(status)) => {
                let _ = started.send(Err(status));
                return;
            }
            Err(panic) => {
                let _ = started.send(Err(panicked(&panic)));
                return;
            }
        };
        if started.send(Ok(())).is_ok() {
            pump(items, &sender);
        }
    }));
    match start_result.await {
        Ok(Ok(())) => Ok(ChannelStream::new(receiver)),
        Ok(Err(status)) => Err(status),
        Err(_) => Err(Status::internal("blocking call ended without a result")),
    }
}

/// Send each item of `items` until it ends, an `Err` item has been sent
/// (an `Err` ends a stream), the reader goes away, or `next` panics (sent
/// as a last `Status::internal` item).
pub(crate) fn pump<X>(
    mut items: BoxIter<'static, Result<X, Status>>,
    sender: &mpsc::Sender<Result<X, Status>>,
) {
    loop {
        let item = match catch_unwind(AssertUnwindSafe(|| items.next())) {
            Ok(Some(item)) => item,
            Ok(None) => return,
            Err(panic) => {
                let _ = sender.blocking_send(Err(panicked(&panic)));
                return;
            }
        };
        let ends = item.is_err();
        if sender.blocking_send(item).is_err() || ends {
            return;
        }
    }
}

/// `Status::internal` for a caught panic, with its message when it has one.
pub(crate) fn panicked(panic: &Box<dyn Any + Send>) -> Status {
    let message = panic
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("no message");
    Status::internal(format!("the implementation panicked: {message}"))
}

/// An async stream read as an iterator on a blocking-pool thread: each
/// `next` waits for the stream's next item.
struct StreamIter<S> {
    handle: Handle,
    stream: std::pin::Pin<Box<S>>,
}

impl<S> StreamIter<S> {
    fn new(handle: Handle, stream: S) -> StreamIter<S> {
        StreamIter {
            handle,
            stream: Box::pin(stream),
        }
    }
}

impl<S: Stream> Iterator for StreamIter<S> {
    type Item = S::Item;

    fn next(&mut self) -> Option<S::Item> {
        self.handle.block_on(next(&mut self.stream))
    }
}
