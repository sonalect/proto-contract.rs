//! `Inline`: the async form of a sync implementation, run in place.

use futures_core::Stream;

use crate::BoxIter;
use crate::stream::next;

/// The async form of a sync implementation, without a thread hop: each call
/// runs the sync method inside the future's `poll`.
///
/// Use it only for calls that return in microseconds: while the sync method
/// runs, the async task cannot do anything else, and on a runtime worker
/// thread neither can the other tasks of that worker. For longer calls use
/// `Offload` (feature `tokio`).
///
/// A reply stream yields the sync iterator's items in place. An inbound
/// stream is read to its end before the sync method is called, because a
/// sync method cannot wait for the next item. A bidirectional call through
/// `Inline` therefore answers only after the caller has sent every request:
/// do not use it for a conversation where the caller waits for a reply
/// before it sends the next request.
///
/// The plugin implements each `<Service>Async` for `Inline<T>` where `T`
/// implements `<Service>Sync`.
#[derive(Clone, Debug, Default)]
pub struct Inline<T> {
    service: T,
}

impl<T> Inline<T> {
    /// The async form of `service`.
    pub fn new(service: T) -> Inline<T> {
        Inline { service }
    }

    /// The sync implementation.
    pub fn get_ref(&self) -> &T {
        &self.service
    }

    /// The sync implementation, moved out.
    pub fn into_inner(self) -> T {
        self.service
    }

    /// Read `requests` to its end and hand the items over as an iterator,
    /// for a sync method that takes a stream.
    pub async fn buffer<S>(requests: S) -> BoxIter<'static, S::Item>
    where
        S: Stream,
        S::Item: Send + 'static,
    {
        let mut requests = Box::pin(requests);
        let mut items = Vec::new();
        while let Some(item) = next(&mut requests).await {
            items.push(item);
        }
        Box::new(items.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};

    use super::Inline;
    use crate::IterStream;

    #[test]
    fn buffer_keeps_every_item_in_order() {
        // An `IterStream` never waits, so one poll finishes the buffering.
        let mut buffering = pin!(Inline::<()>::buffer(IterStream::new(1..=4)));
        let Poll::Ready(items) = buffering
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        else {
            panic!("buffering waited");
        };
        assert_eq!(items.collect::<Vec<_>>(), [1, 2, 3, 4]);
        assert_eq!(Inline::new(5).into_inner(), 5);
    }
}
