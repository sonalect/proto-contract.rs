//! Streams the bridges hand out.

use std::future::poll_fn;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

/// A [`Stream`] over an iterator: each `poll_next` takes the iterator's
/// next item in place, without waiting.
///
/// The iterator runs on the polling thread, so it suits iterators whose
/// `next` returns in microseconds.
///
/// ```
/// use contract::{IterStream, Stream};
///
/// fn numbers() -> impl Stream<Item = u32> {
///     IterStream::new(1..=3)
/// }
/// # let _ = numbers();
/// ```
#[derive(Debug)]
pub struct IterStream<I> {
    iter: I,
}

impl<I> IterStream<I> {
    /// A stream that yields the items of `iter`.
    pub fn new(iter: I) -> IterStream<I> {
        IterStream { iter }
    }
}

impl<I: Iterator + Unpin> Stream for IterStream<I> {
    type Item = I::Item;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<I::Item>> {
        Poll::Ready(self.iter.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// The next item of `stream`.
pub(crate) async fn next<S: Stream + Unpin + ?Sized>(stream: &mut S) -> Option<S::Item> {
    poll_fn(|cx| Pin::new(&mut *stream).poll_next(cx)).await
}

#[cfg(feature = "tokio")]
pub use channel::ChannelStream;

#[cfg(feature = "tokio")]
mod channel {
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use futures_core::Stream;
    use tokio::sync::mpsc::Receiver;

    /// A [`Stream`] of the items another thread sends: the async side of a
    /// stream that `Offload` or `Blocking` moves between threads.
    ///
    /// Dropping it tells the sending side to stop.
    #[derive(Debug)]
    pub struct ChannelStream<T> {
        receiver: Receiver<T>,
    }

    impl<T> ChannelStream<T> {
        pub(crate) fn new(receiver: Receiver<T>) -> ChannelStream<T> {
            ChannelStream { receiver }
        }
    }

    impl<T> Stream for ChannelStream<T> {
        type Item = T;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
            self.receiver.poll_recv(cx)
        }
    }
}
