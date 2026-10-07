//! The bridges between the forms, for every kind of method: `Inline` and
//! `Offload` give a sync implementation the async form, `Blocking` gives an
//! async implementation the sync form, called from every kind of thread.

use std::future::poll_fn;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use contract_golden::proto::example::v1::{Delta, FeedErrorCode, Item, Limit, Summary, Total};
use contract_golden::traits::example::v1::{
    CounterAsync, CounterSync, DynFeedAsync, FeedAsync, FeedSync, ToolsServiceAsync,
    ToolsServiceSync,
};
use protocontract::{Blocking, BoxIter, Error, Inline, IterStream, Offload, RuntimeCode, Stream};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc;

// ---- plumbing ----

/// A stream over a tokio channel, for inbound and reply streams.
struct Channel<T>(mpsc::Receiver<T>);

impl<T> Stream for Channel<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        self.0.poll_recv(cx)
    }
}

async fn next<S: Stream + Unpin>(stream: &mut S) -> Option<S::Item> {
    poll_fn(|cx| Pin::new(&mut *stream).poll_next(cx)).await
}

async fn drain<S: Stream>(stream: S) -> Vec<S::Item> {
    let mut stream = Box::pin(stream);
    let mut items = Vec::new();
    while let Some(item) = next(&mut stream).await {
        items.push(item);
    }
    items
}

fn multi_thread() -> Runtime {
    Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
}

fn current_thread() -> Runtime {
    Builder::new_current_thread().enable_all().build().unwrap()
}

fn item(text: &str) -> Item {
    Item {
        text: text.into(),
        ..Default::default()
    }
}

fn items(texts: &[&str]) -> Vec<Result<Item, Error>> {
    texts.iter().map(|text| Ok(item(text))).collect()
}

fn texts(items: Vec<Result<Item, Error>>) -> Vec<String> {
    items.into_iter().map(|item| item.unwrap().text).collect()
}

fn add(delta: i64) -> Delta {
    Delta {
        value: Some(delta),
        ..Default::default()
    }
}

fn watch(count: i32) -> Limit {
    Limit {
        count,
        ..Default::default()
    }
}

/// Count of `watch` that makes the sync feed panic after its first item.
const PANIC_AFTER_ONE: i32 = 13;

/// Count of `watch` that makes the sync feed yield an `Err` second, then
/// go on as if nothing happened.
const ERR_AT_ONE: i32 = 7;

// ---- the sync implementations ----

#[derive(Default)]
struct SyncCounter {
    total: AtomicI64,
}

impl CounterSync for SyncCounter {
    fn add(&self, request: Delta) -> Result<Total, Error> {
        let delta = request.value.unwrap_or_default();
        if delta == i64::MIN {
            panic!("delta out of range");
        }
        let total = self.total.fetch_add(delta, Ordering::SeqCst) + delta;
        Ok(Total {
            value: Some(total),
            ..Default::default()
        })
    }
}

/// Counts the reply items its streams have produced.
#[derive(Default)]
struct SyncFeed {
    pulled: Arc<AtomicUsize>,
}

impl FeedSync for SyncFeed {
    fn watch(&self, request: Limit) -> Result<BoxIter<'static, Result<Item, Error>>, Error> {
        if request.count < 0 {
            return Err(Error::new(
                FeedErrorCode::NegativeCount,
                "count is negative",
            ));
        }
        let (panics, fails) = (
            request.count == PANIC_AFTER_ONE,
            request.count == ERR_AT_ONE,
        );
        let pulled = Arc::clone(&self.pulled);
        Ok(Box::new((0..request.count).map(move |i| {
            pulled.fetch_add(1, Ordering::SeqCst);
            if panics && i == 1 {
                panic!("feed broke");
            }
            if fails && i == 1 {
                return Err(Error::new(FeedErrorCode::FeedFailed, "feed failed"));
            }
            Ok(item(&format!("s{i}")))
        })))
    }

    fn collect(&self, requests: BoxIter<'static, Result<Item, Error>>) -> Result<Summary, Error> {
        let mut texts = Vec::new();
        for request in requests {
            texts.push(request?.text);
        }
        Ok(Summary {
            count: i32::try_from(texts.len()).unwrap(),
            joined: texts.join(" "),
            ..Default::default()
        })
    }

    fn echo(
        &self,
        requests: BoxIter<'static, Result<Item, Error>>,
    ) -> Result<BoxIter<'static, Result<Item, Error>>, Error> {
        Ok(Box::new(requests.map(|request| {
            request.map(|request| item(&request.text.to_uppercase()))
        })))
    }
}

// ---- the async implementations: they yield, so a runtime must drive them ----

#[derive(Default)]
struct AsyncCounter {
    total: AtomicI64,
}

impl CounterAsync for AsyncCounter {
    async fn add(&self, request: Delta) -> Result<Total, Error> {
        tokio::task::yield_now().await;
        let delta = request.value.unwrap_or_default();
        let total = self.total.fetch_add(delta, Ordering::SeqCst) + delta;
        Ok(Total {
            value: Some(total),
            ..Default::default()
        })
    }
}

struct AsyncFeed;

impl FeedAsync for AsyncFeed {
    async fn watch(
        &self,
        request: Limit,
    ) -> Result<impl Stream<Item = Result<Item, Error>> + Send + use<>, Error> {
        if request.count < 0 {
            return Err(Error::new(
                FeedErrorCode::NegativeCount,
                "count is negative",
            ));
        }
        let (sender, receiver) = mpsc::channel(2);
        tokio::spawn(async move {
            for i in 0..request.count {
                tokio::task::yield_now().await;
                if sender.send(Ok(item(&format!("a{i}")))).await.is_err() {
                    return;
                }
            }
        });
        Ok(Channel(receiver))
    }

    async fn collect<R>(&self, requests: R) -> Result<Summary, Error>
    where
        R: Stream<Item = Result<Item, Error>> + Send + 'static,
    {
        let mut texts = Vec::new();
        for request in drain(requests).await {
            texts.push(request?.text);
        }
        Ok(Summary {
            count: i32::try_from(texts.len()).unwrap(),
            joined: texts.join(" "),
            ..Default::default()
        })
    }

    async fn echo<R>(
        &self,
        requests: R,
    ) -> Result<impl Stream<Item = Result<Item, Error>> + Send + use<R>, Error>
    where
        R: Stream<Item = Result<Item, Error>> + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel(1);
        tokio::spawn(async move {
            let mut requests = Box::pin(requests);
            while let Some(request) = next(&mut requests).await {
                let reply = request.map(|request| item(&request.text.to_uppercase()));
                if sender.send(reply).await.is_err() {
                    return;
                }
            }
        });
        Ok(Channel(receiver))
    }
}

// ---- Inline ----

#[test]
fn inline_runs_every_kind_in_place() {
    let counter = Inline::new(SyncCounter::default());
    let feed = Inline::new(SyncFeed::default());
    current_thread().block_on(async {
        assert_eq!(counter.add(add(4)).await.unwrap().value, Some(4));
        assert_eq!(
            texts(drain(feed.watch(watch(2)).await.unwrap()).await),
            ["s0", "s1"]
        );
        let error = feed.watch(watch(-1)).await.err().unwrap();
        assert!(error.is(FeedErrorCode::NegativeCount));

        let summary = feed
            .collect(IterStream::new(items(&["a", "b"]).into_iter()))
            .await
            .unwrap();
        assert_eq!(summary.joined, "a b");
        let echoed = feed
            .echo(IterStream::new(items(&["x", "y"]).into_iter()))
            .await
            .unwrap();
        assert_eq!(texts(drain(echoed).await), ["X", "Y"]);
    });
}

#[test]
fn inline_fits_in_the_dyn_handle() {
    let service = DynFeedAsync::new(Inline::new(SyncFeed::default()));
    let replies =
        current_thread().block_on(async { drain(service.watch(watch(1)).await.unwrap()).await });
    assert_eq!(texts(replies), ["s0"]);
}

// ---- Offload ----

#[test]
fn offload_runs_every_kind_on_the_blocking_pool() {
    for runtime in [multi_thread(), current_thread()] {
        let counter = Offload::new(SyncCounter::default(), runtime.handle().clone());
        let feed = Offload::new(SyncFeed::default(), runtime.handle().clone());
        runtime.block_on(async {
            assert_eq!(counter.add(add(2)).await.unwrap().value, Some(2));
            assert_eq!(counter.add(add(3)).await.unwrap().value, Some(5));

            let replies = drain(feed.watch(watch(3)).await.unwrap()).await;
            assert_eq!(texts(replies), ["s0", "s1", "s2"]);
            let error = feed.watch(watch(-1)).await.err().unwrap();
            assert!(error.is(FeedErrorCode::NegativeCount));

            let summary = feed
                .collect(IterStream::new(items(&["a", "b", "c"]).into_iter()))
                .await
                .unwrap();
            assert_eq!((summary.count, summary.joined.as_str()), (3, "a b c"));
            let mut broken = items(&["a"]);
            broken.push(Err(Error::new(FeedErrorCode::ConnectionReset, "reset")));
            let error = feed
                .collect(IterStream::new(broken.into_iter()))
                .await
                .unwrap_err();
            assert!(error.is(FeedErrorCode::ConnectionReset));
        });
    }
}

/// A bidirectional call through `Offload` answers each request as it
/// arrives: the caller waits for each reply before it sends the next
/// request.
#[test]
fn offload_bidirectional_is_a_conversation() {
    for runtime in [multi_thread(), current_thread()] {
        let feed = Offload::new(SyncFeed::default(), runtime.handle().clone());
        runtime.block_on(async {
            let (requests, inbound) = mpsc::channel(1);
            let mut replies = Box::pin(feed.echo(Channel(inbound)).await.unwrap());
            for text in ["ping", "pong", "done"] {
                requests.send(Ok(item(text))).await.unwrap();
                let reply = next(&mut replies).await.unwrap().unwrap();
                assert_eq!(reply.text, text.to_uppercase());
            }
            drop(requests);
            assert!(next(&mut replies).await.is_none());
        });
    }
}

/// Dropping a reply stream of `Offload` stops the pull on the blocking
/// pool, within the channel's 16 items.
#[test]
fn offload_stops_pulling_a_dropped_stream() {
    let runtime = multi_thread();
    let feed = SyncFeed::default();
    let pulled = Arc::clone(&feed.pulled);
    let feed = Offload::new(feed, runtime.handle().clone());
    runtime.block_on(async {
        let mut replies = Box::pin(feed.watch(watch(i32::MAX)).await.unwrap());
        for _ in 0..3 {
            next(&mut replies).await.unwrap().unwrap();
        }
        drop(replies);
        tokio::time::sleep(Duration::from_millis(50)).await;
        let settled = pulled.load(Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(pulled.load(Ordering::SeqCst), settled);
        assert!(settled <= 3 + 16 + 2, "pulled {settled} items");
    });
}

/// An `Err` item ends a stream at every bridge, even when the sync
/// iterator goes on after it.
#[test]
fn an_error_item_ends_the_stream() {
    let runtime = multi_thread();
    let offload = Offload::new(SyncFeed::default(), runtime.handle().clone());
    let inline = Inline::new(SyncFeed::default());
    runtime.block_on(async {
        for replies in [
            drain(offload.watch(watch(ERR_AT_ONE)).await.unwrap()).await,
            drain(inline.watch(watch(ERR_AT_ONE)).await.unwrap()).await,
        ] {
            assert_eq!(replies.len(), 2);
            assert_eq!(replies[0].as_ref().unwrap().text, "s0");
            assert!(
                replies[1]
                    .as_ref()
                    .unwrap_err()
                    .is(FeedErrorCode::FeedFailed)
            );
        }
    });
    let blocking = Blocking::new(offload, runtime.handle().clone());
    let replies: Vec<_> = FeedSync::watch(&blocking, watch(ERR_AT_ONE))
        .unwrap()
        .collect();
    assert_eq!(replies.len(), 2);
    assert!(
        replies[1]
            .as_ref()
            .unwrap_err()
            .is(FeedErrorCode::FeedFailed)
    );
}

#[test]
fn offload_turns_a_panic_into_panicked() {
    let runtime = multi_thread();
    let counter = Offload::new(SyncCounter::default(), runtime.handle().clone());
    let feed = Offload::new(SyncFeed::default(), runtime.handle().clone());
    runtime.block_on(async {
        let error = counter.add(add(i64::MIN)).await.unwrap_err();
        assert!(error.is(RuntimeCode::Panicked));
        assert!(error.to_string().contains("delta out of range"), "{error}");

        let replies = drain(feed.watch(watch(PANIC_AFTER_ONE)).await.unwrap()).await;
        assert_eq!(replies.len(), 2);
        assert_eq!(replies[0].as_ref().unwrap().text, "s0");
        let error = replies[1].as_ref().unwrap_err();
        assert!(error.is(RuntimeCode::Panicked));
        assert!(error.to_string().contains("feed broke"), "{error}");
    });
}

// ---- Blocking ----

fn check_blocking_counter(counter: &Blocking<AsyncCounter>, expected: i64) {
    assert_eq!(counter.add(add(1)).unwrap().value, Some(expected));
}

#[test]
fn blocking_runs_every_kind_from_a_plain_thread() {
    let runtime = multi_thread();
    let counter = Blocking::new(AsyncCounter::default(), runtime.handle().clone());
    let feed = Blocking::new(AsyncFeed, runtime.handle().clone());
    std::thread::spawn(move || {
        check_blocking_counter(&counter, 1);

        let replies: Vec<_> = feed.watch(watch(3)).unwrap().collect();
        assert_eq!(texts(replies), ["a0", "a1", "a2"]);
        let error = feed.watch(watch(-1)).err().unwrap();
        assert!(error.is(FeedErrorCode::NegativeCount));

        let summary = feed
            .collect(Box::new(items(&["a", "b"]).into_iter()))
            .unwrap();
        assert_eq!(summary.joined, "a b");

        let echoed: Vec<_> = feed
            .echo(Box::new(items(&["q", "r"]).into_iter()))
            .unwrap()
            .collect();
        assert_eq!(texts(echoed), ["Q", "R"]);
    })
    .join()
    .unwrap();
}

/// A sync caller converses through `Blocking`: its request iterator waits
/// for the caller, which waits for each reply.
#[test]
fn blocking_bidirectional_is_a_conversation() {
    let runtime = multi_thread();
    let feed = Blocking::new(AsyncFeed, runtime.handle().clone());
    let (requests, inbound) = std::sync::mpsc::channel::<Result<Item, Error>>();
    let mut replies = feed.echo(Box::new(inbound.into_iter())).unwrap();
    for text in ["ping", "pong"] {
        requests.send(Ok(item(text))).unwrap();
        assert_eq!(replies.next().unwrap().unwrap().text, text.to_uppercase());
    }
    drop(requests);
    assert!(replies.next().is_none());
}

/// O1: inside a multi-thread runtime, a call through `Blocking` moves off
/// the worker and succeeds, from a task and from `block_on`'s own future.
#[test]
fn blocking_inside_a_multi_thread_runtime() {
    let runtime = multi_thread();
    let counter = Arc::new(Blocking::new(
        AsyncCounter::default(),
        runtime.handle().clone(),
    ));
    let from_task = Arc::clone(&counter);
    runtime.block_on(async move {
        tokio::spawn(async move { check_blocking_counter(&from_task, 1) })
            .await
            .unwrap();
        check_blocking_counter(&counter, 2);
        let worker = Arc::clone(&counter);
        tokio::task::spawn_blocking(move || check_blocking_counter(&worker, 3))
            .await
            .unwrap();
    });
}

/// O1: inside a current-thread runtime, a call through `Blocking` would
/// stop the runtime it waits for; it fails with `FAILED_PRECONDITION`.
#[test]
fn blocking_inside_a_current_thread_runtime_is_refused() {
    let driver = multi_thread();
    let counter = Arc::new(Blocking::new(
        AsyncCounter::default(),
        driver.handle().clone(),
    ));
    let feed = Blocking::new(AsyncFeed, driver.handle().clone());
    let replies = feed.watch(watch(2)).unwrap();

    let runtime = current_thread();
    let from_task = Arc::clone(&counter);
    runtime.block_on(async move {
        let error = counter.add(add(1)).unwrap_err();
        assert!(error.is(RuntimeCode::CannotBlock));
        let error = tokio::spawn(async move { from_task.add(add(1)).unwrap_err() })
            .await
            .unwrap();
        assert!(error.is(RuntimeCode::CannotBlock));

        // A reply stream read there fails the same way, and ends.
        let mut replies = replies;
        let error = replies.next().unwrap().unwrap_err();
        assert!(error.is(RuntimeCode::CannotBlock));
        assert!(replies.next().is_none());
    });
}

// ---- O5: what a call costs through each bridge ----

/// Prints the cost of one unary call: direct, through `Inline`, and through
/// `Offload`. Run with `--ignored --nocapture` on a release build.
#[test]
#[ignore = "a measurement, not a check"]
fn offload_cost() {
    const CALLS: u32 = 20_000;
    let runtime = multi_thread();
    let direct = SyncCounter::default();
    let inline = Inline::new(SyncCounter::default());
    let offload = Offload::new(SyncCounter::default(), runtime.handle().clone());
    runtime.block_on(async {
        let start = Instant::now();
        for _ in 0..CALLS {
            direct.add(add(1)).unwrap();
        }
        let direct_cost = start.elapsed() / CALLS;
        let start = Instant::now();
        for _ in 0..CALLS {
            inline.add(add(1)).await.unwrap();
        }
        let inline_cost = start.elapsed() / CALLS;
        let start = Instant::now();
        for _ in 0..CALLS {
            offload.add(add(1)).await.unwrap();
        }
        let offload_cost = start.elapsed() / CALLS;
        println!(
            "per call: direct {direct_cost:?}, Inline {inline_cost:?}, Offload {offload_cost:?}"
        );
    });
}

// ---- methods named like the bridges' helpers ----

#[derive(Default)]
struct Tools;

fn tool(request: Delta, base: i64) -> Result<Total, Error> {
    Ok(Total {
        value: Some(base + request.value.unwrap_or_default()),
        ..Default::default()
    })
}

impl ToolsServiceSync for Tools {
    fn call(&self, request: Delta) -> Result<Total, Error> {
        tool(request, 100)
    }
    fn feed(&self, request: Delta) -> Result<Total, Error> {
        tool(request, 200)
    }
    fn get_ref(&self, request: Delta) -> Result<Total, Error> {
        tool(request, 300)
    }
    fn block_on(&self, request: Delta) -> Result<Total, Error> {
        tool(request, 400)
    }
    fn into_inner(&self, request: Delta) -> Result<Total, Error> {
        tool(request, 500)
    }
    fn server_streaming(
        &self,
        request: Delta,
    ) -> Result<BoxIter<'static, Result<Total, Error>>, Error> {
        Ok(Box::new(std::iter::once(tool(request, 600))))
    }
}

/// A method named like a bridge helper is reached with method syntax: the
/// helpers are associated functions and hide nothing.
#[test]
fn methods_named_like_helpers_are_reachable() {
    let runtime = multi_thread();
    let inline = Inline::new(Tools);
    let offload = Offload::new(Tools, runtime.handle().clone());
    runtime.block_on(async {
        assert_eq!(inline.get_ref(add(1)).await.unwrap().value, Some(301));
        assert_eq!(offload.call(add(1)).await.unwrap().value, Some(101));
        assert_eq!(offload.feed(add(1)).await.unwrap().value, Some(201));
        let replies = drain(offload.server_streaming(add(1)).await.unwrap()).await;
        assert_eq!(replies[0].as_ref().unwrap().value, Some(601));
    });
    let blocking = Blocking::new(offload, runtime.handle().clone());
    assert_eq!(blocking.block_on(add(1)).unwrap().value, Some(401));
    assert_eq!(blocking.into_inner(add(1)).unwrap().value, Some(501));
    assert_eq!(blocking.get_ref(add(1)).unwrap().value, Some(301));
}
