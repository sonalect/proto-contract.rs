//! Direct calls of unary methods through the generated traits: one
//! implementation per form, called generically, through `Arc<dyn …>`
//! (sync), and through the `Dyn…Async` handle (async).

use std::future::Future;
use std::pin::pin;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use buffa_types::google::protobuf::Empty;
use contract_golden::per_package;
use contract_golden::proto::example::common::v1::Name;
use contract_golden::proto::example::v1::person::Style;
use contract_golden::proto::example::v1::{
    Delta, GreeterErrorCode, Greeting, Person, ToolsErrorCode, Total,
};
use contract_golden::shared::v1::Label;
use contract_golden::traits::example::v1::{
    CounterAsync, CounterSync, DynCounterAsync, DynGreeterAsync, GreeterAsync, GreeterSync,
    ToolsServiceErrorCode,
};
use protocontract::Error;

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

fn text(reply: &Greeting) -> &str {
    reply.text.as_deref().unwrap_or_default()
}

fn greeting(name: &Name, style: &Style) -> Result<Greeting, Error> {
    if name.first.is_empty() {
        return Err(Error::new(
            GreeterErrorCode::EmptyName,
            "name.first is empty",
        ));
    }
    let text = format!("Hello, {}!", name.first);
    let text = if style.shout == Some(true) {
        text.to_uppercase()
    } else {
        text
    };
    Ok(Greeting {
        text: Some(text),
        ..Default::default()
    })
}

fn request(first: &str, shout: bool) -> Person {
    Person {
        name: Name {
            first: first.into(),
            ..Default::default()
        }
        .into(),
        style: Style {
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

impl GreeterSync for SyncGreeter {
    fn greet(&self, request: Person) -> Result<Greeting, Error> {
        greeting(&request.name, &request.style)
    }

    fn greet_name(&self, request: Name) -> Result<Greeting, Error> {
        greeting(&request, &Style::default())
    }

    fn get_style(&self, _request: Empty) -> Result<Style, Error> {
        Ok(Style {
            shout: Some(true),
            ..Default::default()
        })
    }

    fn echo_label(&self, request: Label) -> Result<Label, Error> {
        Ok(request)
    }

    fn reset(&self, request: Empty) -> Result<Empty, Error> {
        self.resets.fetch_add(1, Ordering::SeqCst);
        Ok(request)
    }
}

/// The async form, written with plain `async fn`.
#[derive(Default)]
struct AsyncGreeter {
    log: Mutex<Vec<String>>,
}

impl GreeterAsync for AsyncGreeter {
    async fn greet(&self, request: Person) -> Result<Greeting, Error> {
        let reply = greeting(&request.name, &request.style)?;
        self.log.lock().unwrap().push(text(&reply).to_string());
        Ok(reply)
    }

    async fn greet_name(&self, request: Name) -> Result<Greeting, Error> {
        greeting(&request, &Style::default())
    }

    async fn get_style(&self, _request: Empty) -> Result<Style, Error> {
        Ok(Style::default())
    }

    async fn echo_label(&self, request: Label) -> Result<Label, Error> {
        Ok(request)
    }

    async fn reset(&self, _request: Empty) -> Result<Empty, Error> {
        Err(Error::new(
            GreeterErrorCode::NotSupported,
            "reset is not supported",
        ))
    }
}

/// A caller generic over the async form: no boxing.
async fn greet_twice(service: &impl GreeterAsync, first: &str) -> Vec<String> {
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
    let service: Arc<dyn GreeterSync> = greeter.clone();

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
        service.get_style(Empty::default()).unwrap().shout,
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
    let service: Arc<dyn GreeterSync> = Arc::new(SyncGreeter::default());
    let error = service.greet(request("", false)).unwrap_err();
    assert!(error.is(GreeterErrorCode::EmptyName));
    // A caller matches on the enum's constants, as the README shows.
    let what = match error.code_as::<GreeterErrorCode>() {
        Some(GreeterErrorCode::EmptyName) => "empty name",
        Some(_) => "another greeter code",
        None => "not a greeter code",
    };
    assert_eq!(what, "empty name");
    assert_eq!(
        error.to_string(),
        "GREETER_ERROR_CODE_EMPTY_NAME: name.first is empty"
    );
}

/// `ToolsService` keeps its `Service` suffix, so the plugin emits the alias
/// `ToolsServiceErrorCode` for the enum `ToolsErrorCode`.
#[test]
fn error_code_alias_names_the_enum() {
    let error = Error::new(ToolsErrorCode::Broken, "a tool broke");
    assert!(error.is(ToolsServiceErrorCode::Broken));
    assert!(error.is(ToolsServiceErrorCode::TOOLS_ERROR_CODE_BROKEN));
    assert_eq!(
        error.code_as::<ToolsServiceErrorCode>(),
        Some(ToolsErrorCode::Broken)
    );
    let what = match error.code_as::<ToolsServiceErrorCode>() {
        Some(ToolsServiceErrorCode::Broken) => "broken",
        Some(_) => "another tools code",
        None => "not a tools code",
    };
    assert_eq!(what, "broken");
    // Number 1 of another enum is another code.
    assert_eq!(
        GreeterErrorCode::EmptyName as i32,
        ToolsErrorCode::Broken as i32
    );
    assert!(!error.is(GreeterErrorCode::EmptyName));
    assert_eq!(error.code_as::<GreeterErrorCode>(), None);
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
    let service = DynGreeterAsync::from_arc(Arc::clone(&greeter));

    assert_eq!(
        text(&ready(service.greet(request("Ada", true))).unwrap()),
        "HELLO, ADA!"
    );
    assert_eq!(
        text(&ready(service.greet_name(name("Grace"))).unwrap()),
        "Hello, Grace!"
    );
    assert_eq!(
        ready(service.get_style(Empty::default())).unwrap().shout,
        None
    );
    assert_eq!(*greeter.log.lock().unwrap(), ["HELLO, ADA!"]);

    let error = ready(service.reset(Empty::default())).unwrap_err();
    assert!(error.is(GreeterErrorCode::NotSupported));
    let error = ready(service.greet(request("", false))).unwrap_err();
    assert!(error.is(GreeterErrorCode::EmptyName));
    // A caller matches on the enum's constants, as the README shows.
    let what = match error.code_as::<GreeterErrorCode>() {
        Some(GreeterErrorCode::EmptyName) => "empty name",
        Some(_) => "another greeter code",
        None => "not a greeter code",
    };
    assert_eq!(what, "empty name");
}

#[test]
fn dyn_handle_is_an_implementation_and_moves_across_threads() {
    let service = DynGreeterAsync::new(AsyncGreeter::default());
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
    let service = DynGreeterAsync::new(AsyncGreeter::default());
    assert_eq!(format!("{service:?}"), "DynGreeterAsync { .. }");
}

/// The handle may be chosen at run time among implementations.
#[test]
fn dyn_handle_swaps_implementations() {
    struct Polite;
    impl GreeterAsync for Polite {
        async fn greet(&self, _request: Person) -> Result<Greeting, Error> {
            Ok(Greeting {
                text: Some("Good day.".into()),
                ..Default::default()
            })
        }
        async fn greet_name(&self, _request: Name) -> Result<Greeting, Error> {
            Err(Error::new(GreeterErrorCode::NotSupported, "greet_name"))
        }
        async fn get_style(&self, _request: Empty) -> Result<Style, Error> {
            Err(Error::new(GreeterErrorCode::NotSupported, "get_style"))
        }
        async fn echo_label(&self, request: Label) -> Result<Label, Error> {
            Ok(request)
        }
        async fn reset(&self, request: Empty) -> Result<Empty, Error> {
            Ok(request)
        }
    }

    for (polite, expected) in [(true, "Good day."), (false, "Hello, Ada!")] {
        let service = if polite {
            DynGreeterAsync::new(Polite)
        } else {
            DynGreeterAsync::new(AsyncGreeter::default())
        };
        let reply = ready(service.greet(request("Ada", false))).unwrap();
        assert_eq!(text(&reply), expected);
    }
}

/// One type may implement both forms; each call names the form it uses.
struct Counter {
    total: AtomicI64,
}

impl CounterSync for Counter {
    fn add(&self, request: Delta) -> Result<Total, Error> {
        let delta = request.value.unwrap_or_default();
        let total = self.total.fetch_add(delta, Ordering::SeqCst) + delta;
        Ok(Total {
            value: Some(total),
            ..Default::default()
        })
    }
}

impl CounterAsync for Counter {
    async fn add(&self, request: Delta) -> Result<Total, Error> {
        CounterSync::add(self, request)
    }
}

#[test]
fn one_type_may_implement_both_forms() {
    let counter = Arc::new(Counter {
        total: AtomicI64::new(0),
    });
    let sync: Arc<dyn CounterSync> = counter.clone();
    let asynchronous = DynCounterAsync::from_arc(counter);
    let add = |delta| Delta {
        value: Some(delta),
        ..Default::default()
    };
    assert_eq!(sync.add(add(2)).unwrap().value, Some(2));
    assert_eq!(ready(asynchronous.add(add(-5))).unwrap().value, Some(-3));
}

/// The `file_per_package` layout yields the same traits.
#[test]
fn per_package_layout_is_usable() {
    struct Echo;
    impl per_package::example::v1::CounterSync for Echo {
        fn add(&self, request: Delta) -> Result<Total, Error> {
            Ok(Total {
                value: request.value,
                ..Default::default()
            })
        }
    }
    let service: Box<dyn per_package::example::v1::CounterSync> = Box::new(Echo);
    let reply = service
        .add(Delta {
            value: Some(7),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(reply.value, Some(7));
}
