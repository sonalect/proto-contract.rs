//! Direct calls of unary methods through the generated traits: one
//! implementation per form, called generically, through `Arc<dyn …>`
//! (sync), and through the `Dyn…Async` handle (async).

use std::future::Future;
use std::pin::pin;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use buffa_types::google::protobuf::Empty;
use contract::{Code, Status};
use contract_golden::per_package;
use contract_golden::proto::example::common::v1::Name;
use contract_golden::proto::example::v1::greet_request::Options;
use contract_golden::proto::example::v1::{AddReply, AddRequest, GreetReply, GreetRequest};
use contract_golden::shared::v1::Label;
use contract_golden::traits::example::v1::{
    CounterServiceAsync, CounterServiceSync, DynCounterServiceAsync, DynGreeterServiceAsync,
    GreeterServiceAsync, GreeterServiceSync,
};

/// Drive a future that never waits, as the in-memory implementations here
/// do; no async runtime is needed for them.
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

fn text(reply: &GreetReply) -> &str {
    reply.text.as_deref().unwrap_or_default()
}

fn greeting(name: &Name, options: &Options) -> Result<GreetReply, Status> {
    if name.first.is_empty() {
        return Err(Status::invalid_argument("name.first is empty"));
    }
    let text = format!("Hello, {}!", name.first);
    let text = if options.shout == Some(true) {
        text.to_uppercase()
    } else {
        text
    };
    Ok(GreetReply {
        text: Some(text),
        ..Default::default()
    })
}

fn request(first: &str, shout: bool) -> GreetRequest {
    GreetRequest {
        name: Name {
            first: first.into(),
            ..Default::default()
        }
        .into(),
        options: Options {
            shout: Some(shout),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    }
}

fn name(first: &str) -> Name {
    Name {
        first: first.into(),
        ..Default::default()
    }
}

/// The sync form: computes and returns.
#[derive(Default)]
struct SyncGreeter {
    resets: AtomicI64,
}

impl GreeterServiceSync for SyncGreeter {
    fn greet(&self, request: GreetRequest) -> Result<GreetReply, Status> {
        greeting(&request.name, &request.options)
    }

    fn greet_name(&self, request: Name) -> Result<GreetReply, Status> {
        greeting(&request, &Options::default())
    }

    fn get_options(&self, _request: Empty) -> Result<Options, Status> {
        Ok(Options {
            shout: Some(true),
            ..Default::default()
        })
    }

    fn echo_label(&self, request: Label) -> Result<Label, Status> {
        Ok(request)
    }

    fn reset(&self, request: Empty) -> Result<Empty, Status> {
        self.resets.fetch_add(1, Ordering::SeqCst);
        Ok(request)
    }
}

/// The async form, written with plain `async fn`.
#[derive(Default)]
struct AsyncGreeter {
    log: Mutex<Vec<String>>,
}

impl GreeterServiceAsync for AsyncGreeter {
    async fn greet(&self, request: GreetRequest) -> Result<GreetReply, Status> {
        let reply = greeting(&request.name, &request.options)?;
        self.log.lock().unwrap().push(text(&reply).to_string());
        Ok(reply)
    }

    async fn greet_name(&self, request: Name) -> Result<GreetReply, Status> {
        greeting(&request, &Options::default())
    }

    async fn get_options(&self, _request: Empty) -> Result<Options, Status> {
        Ok(Options::default())
    }

    async fn echo_label(&self, request: Label) -> Result<Label, Status> {
        Ok(request)
    }

    async fn reset(&self, _request: Empty) -> Result<Empty, Status> {
        Err(Status::unimplemented("reset is not supported"))
    }
}

/// A caller generic over the async form: no boxing.
async fn greet_twice(service: &impl GreeterServiceAsync, first: &str) -> Vec<String> {
    let mut texts = Vec::new();
    for shout in [false, true] {
        let reply = service.greet(request(first, shout)).await.unwrap();
        texts.push(text(&reply).to_string());
    }
    texts
}

#[test]
fn sync_implementation_through_dyn() {
    let greeter = Arc::new(SyncGreeter::default());
    let service: Arc<dyn GreeterServiceSync> = greeter.clone();

    assert_eq!(
        text(&service.greet(request("Ada", false)).unwrap()),
        "Hello, Ada!"
    );
    assert_eq!(
        text(&service.greet(request("Ada", true)).unwrap()),
        "HELLO, ADA!"
    );
    assert_eq!(
        text(&service.greet_name(name("Grace")).unwrap()),
        "Hello, Grace!"
    );
    assert_eq!(
        service.get_options(Empty::default()).unwrap().shout,
        Some(true)
    );
    let label = Label {
        text: "vip".into(),
        ..Default::default()
    };
    assert_eq!(service.echo_label(label.clone()).unwrap(), label);
    service.reset(Empty::default()).unwrap();
    assert_eq!(greeter.resets.load(Ordering::SeqCst), 1);
}

#[test]
fn sync_error_keeps_code_and_message() {
    let service: Arc<dyn GreeterServiceSync> = Arc::new(SyncGreeter::default());
    let status = service.greet(request("", false)).unwrap_err();
    assert_eq!(status.code(), Code::InvalidArgument);
    assert_eq!(status.message(), "name.first is empty");
}

#[test]
fn async_implementation_called_generically() {
    let greeter = AsyncGreeter::default();
    assert_eq!(
        ready(greet_twice(&greeter, "Ada")),
        ["Hello, Ada!", "HELLO, ADA!"]
    );
}

#[test]
fn async_implementation_through_the_dyn_handle() {
    let greeter = Arc::new(AsyncGreeter::default());
    let service = DynGreeterServiceAsync::from_arc(Arc::clone(&greeter));

    assert_eq!(
        text(&ready(service.greet(request("Ada", true))).unwrap()),
        "HELLO, ADA!"
    );
    assert_eq!(
        text(&ready(service.greet_name(name("Grace"))).unwrap()),
        "Hello, Grace!"
    );
    assert_eq!(
        ready(service.get_options(Empty::default())).unwrap().shout,
        None
    );
    assert_eq!(*greeter.log.lock().unwrap(), ["HELLO, ADA!"]);

    let status = ready(service.reset(Empty::default())).unwrap_err();
    assert_eq!(status.code(), Code::Unimplemented);
    let status = ready(service.greet(request("", false))).unwrap_err();
    assert_eq!(status.code(), Code::InvalidArgument);
}

#[test]
fn dyn_handle_is_an_implementation_and_moves_across_threads() {
    let service = DynGreeterServiceAsync::new(AsyncGreeter::default());
    // The handle implements the trait, so generic callers take it too.
    assert_eq!(
        ready(greet_twice(&service, "Bo")),
        ["Hello, Bo!", "HELLO, BO!"]
    );

    let worker = service.clone();
    let reply = std::thread::spawn(move || ready(worker.greet(request("Ada", false))))
        .join()
        .unwrap()
        .unwrap();
    assert_eq!(text(&reply), "Hello, Ada!");
}

#[test]
fn dyn_handle_has_debug() {
    let service = DynGreeterServiceAsync::new(AsyncGreeter::default());
    assert_eq!(format!("{service:?}"), "DynGreeterServiceAsync { .. }");
}

/// The handle may be chosen at run time among implementations.
#[test]
fn dyn_handle_swaps_implementations() {
    struct Polite;
    impl GreeterServiceAsync for Polite {
        async fn greet(&self, _request: GreetRequest) -> Result<GreetReply, Status> {
            Ok(GreetReply {
                text: Some("Good day.".into()),
                ..Default::default()
            })
        }
        async fn greet_name(&self, _request: Name) -> Result<GreetReply, Status> {
            Err(Status::unimplemented("greet_name"))
        }
        async fn get_options(&self, _request: Empty) -> Result<Options, Status> {
            Err(Status::unimplemented("get_options"))
        }
        async fn echo_label(&self, request: Label) -> Result<Label, Status> {
            Ok(request)
        }
        async fn reset(&self, request: Empty) -> Result<Empty, Status> {
            Ok(request)
        }
    }

    for (polite, expected) in [(true, "Good day."), (false, "Hello, Ada!")] {
        let service = if polite {
            DynGreeterServiceAsync::new(Polite)
        } else {
            DynGreeterServiceAsync::new(AsyncGreeter::default())
        };
        let reply = ready(service.greet(request("Ada", false))).unwrap();
        assert_eq!(text(&reply), expected);
    }
}

/// One type may implement both forms; each call names the form it uses.
struct Counter {
    total: AtomicI64,
}

impl CounterServiceSync for Counter {
    fn add(&self, request: AddRequest) -> Result<AddReply, Status> {
        let delta = request.delta.unwrap_or_default();
        let total = self.total.fetch_add(delta, Ordering::SeqCst) + delta;
        Ok(AddReply {
            total: Some(total),
            ..Default::default()
        })
    }
}

impl CounterServiceAsync for Counter {
    async fn add(&self, request: AddRequest) -> Result<AddReply, Status> {
        CounterServiceSync::add(self, request)
    }
}

#[test]
fn one_type_may_implement_both_forms() {
    let counter = Arc::new(Counter {
        total: AtomicI64::new(0),
    });
    let sync: Arc<dyn CounterServiceSync> = counter.clone();
    let asynchronous = DynCounterServiceAsync::from_arc(counter);
    let add = |delta| AddRequest {
        delta: Some(delta),
        ..Default::default()
    };
    assert_eq!(sync.add(add(2)).unwrap().total, Some(2));
    assert_eq!(ready(asynchronous.add(add(-5))).unwrap().total, Some(-3));
}

/// The `file_per_package` layout yields the same traits.
#[test]
fn per_package_layout_is_usable() {
    struct Echo;
    impl per_package::example::v1::CounterServiceSync for Echo {
        fn add(&self, request: AddRequest) -> Result<AddReply, Status> {
            Ok(AddReply {
                total: request.delta,
                ..Default::default()
            })
        }
    }
    let service: Box<dyn per_package::example::v1::CounterServiceSync> = Box::new(Echo);
    let reply = service
        .add(AddRequest {
            delta: Some(7),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(reply.total, Some(7));
}
