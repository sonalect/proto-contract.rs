//! `Inline`: the async form of a sync implementation, run in place.

use futures_core::Stream;

use crate::stream::next;
use crate::{BoxIter, Error, IterStream};

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
/// A reply stream ends after its first `Err` item.
///
/// The plugin implements each `<Service>Async` for `Inline<T>` where `T`
/// implements `<Service>Sync`. The helpers below are associated functions
/// (`Inline::get_ref(&inline)`), so they never hide a trait method of the
/// same name.
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
    pub fn get_ref(this: &Inline<T>) -> &T {
        &this.service
    }

    /// The sync implementation, moved out.
    pub fn into_inner(this: Inline<T>) -> T {
        this.service
    }

    /// A sync reply iterator as a stream read in `poll`, ended after its
    /// first `Err` item.
    pub fn reply_stream<X: Send + 'static>(
        replies: BoxIter<'static, Result<X, Error>>,
    ) -> IterStream<BoxIter<'static, Result<X, Error>>> {
        IterStream::new(Box::new(UntilError {
            items: Some(replies),
        }))
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

/// An iterator that ends after its first `Err` item.
struct UntilError<I> {
    /// `None` once an `Err` has been yielded.
    items: Option<I>,
}

impl<X, I: Iterator<Item = Result<X, Error>>> Iterator for UntilError<I> {
    type Item = Result<X, Error>;

    fn next(&mut self) -> Option<Result<X, Error>> {
        let item = self.items.as_mut()?.next();
        if matches!(item, Some(Err(_))) {
            self.items = None;
        }
        item
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};

    use super::Inline;
    use crate::{Error, IterStream, RuntimeCode, Stream};

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
        assert_eq!(Inline::into_inner(Inline::new(5)), 5);
    }

    #[test]
    fn reply_stream_ends_after_the_first_error() {
        let replies: Vec<Result<u8, Error>> = vec![
            Ok(1),
            Err(Error::new(RuntimeCode::Cancelled, "stop")),
            Ok(2),
        ];
        let stream = Inline::<()>::reply_stream(Box::new(replies.into_iter()));
        let mut stream = pin!(stream);
        let mut seen = Vec::new();
        while let Poll::Ready(Some(item)) = stream
            .as_mut()
            .poll_next(&mut Context::from_waker(Waker::noop()))
        {
            seen.push(item.map_err(|error| error.to_string()));
        }
        assert_eq!(seen, [Ok(1), Err("CANCELLED: stop".to_owned())]);
    }
}
