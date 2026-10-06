//! Direct calls of streaming methods: server, client, and bidirectional
//! streaming, in both forms, called generically, through
//! `Arc<dyn …Sync>`, and through the `Dyn…Async` handle.

use std::future::{Future, poll_fn};
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

use contratto::{BoxIter, Code, Status, Stream};
use contratto_golden::contract::example::v1::{
    DynFeedServiceAsync, FeedServiceAsync, FeedServiceSync,
};
use contratto_golden::proto::example::v1::{Item, Summary, WatchRequest};

// ---- test plumbing: streams that never wait, polled without a runtime ----

/// A stream over an iterator.
struct Iter<I>(I);

impl<I: Iterator + Unpin> Stream for Iter<I> {
    type Item = I::Item;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<I::Item>> {
        Poll::Ready(self.0.next())
    }
}

/// A stream with `f` applied to each item.
struct Map<S, F>(S, F);

impl<S: Stream + Unpin, T, F: FnMut(S::Item) -> T + Unpin> Stream for Map<S, F> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        let this = &mut *self;
        Pin::new(&mut this.0)
            .poll_next(cx)
            .map(|item| item.map(&mut this.1))
    }
}

async fn next<S: Stream + Unpin>(stream: &mut S) -> Option<S::Item> {
    poll_fn(|cx| Pin::new(&mut *stream).poll_next(cx)).await
}

fn ready<T>(future: impl Future<Output = T>) -> T {
    let mut future = pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("the future waited"),
    }
}

fn drain<S: Stream>(stream: S) -> Vec<S::Item> {
    let mut stream = Box::pin(stream);
    ready(async move {
        let mut items = Vec::new();
        while let Some(item) = next(&mut stream).await {
            items.push(item);
        }
        items
    })
}

fn item(text: &str) -> Item {
    Item {
        text: text.into(),
        ..Default::default()
    }
}

fn texts(items: Vec<Result<Item, Status>>) -> Vec<String> {
    items.into_iter().map(|item| item.unwrap().text).collect()
}

fn watch(count: i32) -> WatchRequest {
    WatchRequest {
        count,
        ..Default::default()
    }
}

/// Inbound items, the last one a broken stream.
fn broken_inbound() -> Vec<Result<Item, Status>> {
    vec![Ok(item("a")), Err(Status::unavailable("connection reset"))]
}

fn summary(requests: impl Iterator<Item = Result<Item, Status>>) -> Result<Summary, Status> {
    let mut texts = Vec::new();
    for request in requests {
        texts.push(request?.text);
    }
    Ok(Summary {
        count: i32::try_from(texts.len()).map_err(|_| Status::out_of_range("too many items"))?,
        joined: texts.join(" "),
        ..Default::default()
    })
}

fn check_count(count: i32) -> Result<(), Status> {
    if count < 0 {
        Err(Status::invalid_argument("count is negative"))
    } else {
        Ok(())
    }
}

// ---- the sync form: iterators, which may borrow the service ----

struct SyncFeed {
    prefix: String,
}

impl FeedServiceSync for SyncFeed {
    fn watch(&self, request: WatchRequest) -> Result<BoxIter<'_, Result<Item, Status>>, Status> {
        check_count(request.count)?;
        let prefix = &self.prefix;
        Ok(Box::new(
            (0..request.count).map(move |i| Ok(item(&format!("{prefix}{i}")))),
        ))
    }

    fn collect(&self, requests: BoxIter<'_, Result<Item, Status>>) -> Result<Summary, Status> {
        summary(requests)
    }

    fn echo<'a>(
        &'a self,
        requests: BoxIter<'a, Result<Item, Status>>,
    ) -> Result<BoxIter<'a, Result<Item, Status>>, Status> {
        Ok(Box::new(requests.map(move |request| {
            request.map(|request| item(&format!("{}{}", self.prefix, request.text)))
        })))
    }
}

#[test]
fn sync_streaming_through_dyn() {
    let service: Arc<dyn FeedServiceSync> = Arc::new(SyncFeed { prefix: "s".into() });

    let replies: Vec<_> = service.watch(watch(3)).unwrap().collect();
    assert_eq!(texts(replies), ["s0", "s1", "s2"]);
    let status = service.watch(watch(-1)).err().unwrap();
    assert_eq!(status.code(), Code::InvalidArgument);

    let requests = Box::new([item("a"), item("b")].into_iter().map(Ok));
    let collected = service.collect(requests).unwrap();
    assert_eq!((collected.count, collected.joined.as_str()), (2, "a b"));
    let status = service
        .collect(Box::new(broken_inbound().into_iter()))
        .unwrap_err();
    assert_eq!(status.code(), Code::Unavailable);

    let echoed: Vec<_> = service
        .echo(Box::new(broken_inbound().into_iter()))
        .unwrap()
        .collect();
    assert_eq!(echoed[0].as_ref().unwrap().text, "sa");
    assert_eq!(echoed[1].as_ref().unwrap_err().code(), Code::Unavailable);
}

// ---- the async form: plain `async fn`, streams that do not borrow it ----

struct AsyncFeed {
    prefix: String,
}

impl FeedServiceAsync for AsyncFeed {
    async fn watch(
        &self,
        request: WatchRequest,
    ) -> Result<impl Stream<Item = Result<Item, Status>> + Send + use<>, Status> {
        check_count(request.count)?;
        let prefix = self.prefix.clone();
        Ok(Iter(
            (0..request.count).map(move |i| Ok(item(&format!("{prefix}{i}")))),
        ))
    }

    async fn collect<R>(&self, requests: R) -> Result<Summary, Status>
    where
        R: Stream<Item = Result<Item, Status>> + Send + 'static,
    {
        let mut requests = Box::pin(requests);
        let mut items = Vec::new();
        while let Some(request) = next(&mut requests).await {
            items.push(request);
        }
        summary(items.into_iter())
    }

    async fn echo<R>(
        &self,
        requests: R,
    ) -> Result<impl Stream<Item = Result<Item, Status>> + Send + use<R>, Status>
    where
        R: Stream<Item = Result<Item, Status>> + Send + 'static,
    {
        let prefix = self.prefix.clone();
        Ok(Map(
            Box::pin(requests),
            move |request: Result<Item, Status>| {
                request.map(|request| item(&format!("{prefix}{}", request.text)))
            },
        ))
    }
}

/// A caller generic over the async form: no boxing anywhere.
async fn round_trip(service: &impl FeedServiceAsync) -> (Vec<String>, Summary, Vec<String>) {
    let watched = drain(service.watch(watch(2)).await.unwrap());
    let collected = service
        .collect(Iter([item("x"), item("y")].into_iter().map(Ok)))
        .await
        .unwrap();
    let echoed = drain(
        service
            .echo(Iter([item("z")].into_iter().map(Ok)))
            .await
            .unwrap(),
    );
    (texts(watched), collected, texts(echoed))
}

#[test]
fn async_streaming_called_generically() {
    let (watched, collected, echoed) = ready(round_trip(&AsyncFeed { prefix: "g".into() }));
    assert_eq!(watched, ["g0", "g1"]);
    assert_eq!((collected.count, collected.joined.as_str()), (2, "x y"));
    assert_eq!(echoed, ["gz"]);
}

#[test]
fn async_streaming_through_the_dyn_handle() {
    let service = DynFeedServiceAsync::new(AsyncFeed { prefix: "d".into() });

    let (watched, collected, echoed) = ready(round_trip(&service));
    assert_eq!(watched, ["d0", "d1"]);
    assert_eq!(collected.joined, "x y");
    assert_eq!(echoed, ["dz"]);

    let status = ready(service.watch(watch(-1))).err().unwrap();
    assert_eq!(status.code(), Code::InvalidArgument);
    let status = ready(service.collect(Iter(broken_inbound().into_iter()))).unwrap_err();
    assert_eq!(status.code(), Code::Unavailable);
    let echoed = drain(ready(service.echo(Iter(broken_inbound().into_iter()))).unwrap());
    assert_eq!(echoed[0].as_ref().unwrap().text, "da");
    assert_eq!(echoed[1].as_ref().unwrap_err().code(), Code::Unavailable);
}

/// A reply stream does not borrow the service: it outlives the handle and
/// moves to another thread.
#[test]
fn reply_stream_outlives_the_service() {
    let service = DynFeedServiceAsync::new(AsyncFeed { prefix: "t".into() });
    let stream = ready(service.watch(watch(2))).unwrap();
    drop(service);
    let replies = std::thread::spawn(move || drain(stream)).join().unwrap();
    assert_eq!(texts(replies), ["t0", "t1"]);

    let stream = {
        let service = AsyncFeed { prefix: "u".into() };
        ready(service.echo(Iter([item("v")].into_iter().map(Ok)))).unwrap()
    };
    assert_eq!(texts(drain(stream)), ["uv"]);
}
