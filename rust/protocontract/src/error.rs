//! The error every generated method returns.

use std::backtrace::Backtrace;
use std::fmt;
use std::sync::Arc;

use crate::{Code, ErrorCode};

/// A failure an implementation reports: what a type must say to travel as
/// an [`Error`].
///
/// [`Failure`] covers the common case. A type of its own fits when the
/// failure carries more (a path, a limit, an identifier) or decides
/// [`Fault::retryable`] from its code; any `Fault` converts into an
/// `Error` with `?`.
///
/// ```
/// use protocontract::{Code, Error, Fault, RuntimeCode};
///
/// #[derive(Debug)]
/// struct Busy {
///     queue: usize,
/// }
///
/// impl std::fmt::Display for Busy {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "{} calls are waiting", self.queue)
///     }
/// }
///
/// impl std::error::Error for Busy {}
///
/// impl Fault for Busy {
///     fn code(&self) -> Code {
///         // A real contract names a code of its own here.
///         Code::of(RuntimeCode::Cancelled)
///     }
///     fn retryable(&self) -> bool {
///         true
///     }
/// }
///
/// let error = Error::from(Busy { queue: 7 });
/// assert!(error.retryable());
/// assert_eq!(error.downcast_ref::<Busy>().map(|busy| busy.queue), Some(7));
/// ```
pub trait Fault: std::error::Error + Send + Sync + 'static {
    /// The class of the failure, which a caller branches on.
    fn code(&self) -> Code;

    /// Whether the same call with the same request may succeed later, as
    /// when a resource is busy or a connection is down for now. `false`
    /// unless the type says otherwise.
    fn retryable(&self) -> bool {
        false
    }

    /// Where the failure was made, when the type captured it.
    fn backtrace(&self) -> Option<&Backtrace> {
        None
    }
}

/// The failure of a call, as every generated method returns it.
///
/// It holds a [`Fault`] (shared, so cloning is cheap) and the stack of
/// operations the error passed on its way out, outermost first. The
/// caller reads:
///
/// - [`Error::code`], [`Error::is`], [`Error::code_as`]: the class of the
///   failure, to branch on;
/// - [`Error::retryable`]: whether to try the same call again later;
/// - [`Error::stack`] and [`Error::backtrace`]: where it failed;
/// - `Display` and [`std::error::Error::source`]: the message and the
///   chain of causes.
///
/// ```
/// use protocontract::{Context, Error, RuntimeCode};
///
/// fn read() -> Result<(), Error> {
///     Err(Error::new(RuntimeCode::Cancelled, "the runtime stopped"))
/// }
///
/// let error = read().context("load").unwrap_err();
/// assert!(error.is(RuntimeCode::Cancelled));
/// assert_eq!(error.stack(), ["load"]);
/// assert_eq!(error.to_string(), "load: CANCELLED: the runtime stopped");
/// ```
#[derive(Clone, Debug)]
pub struct Error {
    fault: Arc<dyn Fault>,
    stack: Vec<String>,
}

impl Error {
    /// A [`Failure`] with `code` and `message`, not retryable and without
    /// a source.
    #[must_use]
    pub fn new(code: impl ErrorCode, message: impl Into<String>) -> Error {
        Failure::new(code, message).into()
    }

    /// The class of the failure.
    #[must_use]
    pub fn code(&self) -> Code {
        self.fault.code()
    }

    /// The code as a `C`, or `None` when it is a code of another type.
    #[must_use]
    pub fn code_as<C: ErrorCode>(&self) -> Option<C> {
        self.code().get()
    }

    /// Whether the code is `code`.
    #[must_use]
    pub fn is<C: ErrorCode>(&self, code: C) -> bool {
        self.code().is(code)
    }

    /// Whether the same call with the same request may succeed later.
    #[must_use]
    pub fn retryable(&self) -> bool {
        self.fault.retryable()
    }

    /// The operations the error passed on its way out, outermost first.
    #[must_use]
    pub fn stack(&self) -> &[String] {
        &self.stack
    }

    /// Where the failure was made, when its fault captured it. [`Failure`]
    /// captures one when `RUST_BACKTRACE` or `RUST_LIB_BACKTRACE` is set.
    #[must_use]
    pub fn backtrace(&self) -> Option<&Backtrace> {
        self.fault.backtrace()
    }

    /// The fault, when it is a `F`: the way to the fields of an
    /// implementation's own fault type. A caller that branches on them is
    /// bound to that implementation, not to the contract; branch on the
    /// code instead.
    #[must_use]
    pub fn downcast_ref<F: Fault>(&self) -> Option<&F> {
        let fault: &(dyn std::error::Error + 'static) = &*self.fault;
        fault.downcast_ref()
    }

    /// This error with `frame` as its new outermost operation. An empty
    /// frame is skipped.
    #[must_use]
    pub fn context(mut self, frame: impl fmt::Display) -> Error {
        let frame = frame.to_string();
        if !frame.is_empty() {
            self.stack.insert(0, frame);
        }
        self
    }
}

impl<F: Fault> From<F> for Error {
    fn from(fault: F) -> Error {
        Error {
            fault: Arc::new(fault),
            stack: Vec::new(),
        }
    }
}

impl fmt::Display for Error {
    /// `frame: frame: CODE: message`, outermost frame first.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for frame in &self.stack {
            write!(f, "{frame}: ")?;
        }
        write!(f, "{}: {}", self.fault.code(), self.fault)
    }
}

impl std::error::Error for Error {
    /// The cause of the fault: `Display` already shows the fault itself.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.fault.source()
    }
}

/// The ready-made [`Fault`]: a code, a message, whether to retry, an
/// optional cause, and a backtrace.
///
/// ```
/// use protocontract::{Error, Failure, RuntimeCode};
///
/// let cause = std::io::Error::other("connection reset");
/// let error: Error = Failure::new(RuntimeCode::Cancelled, "the feed broke")
///     .with_retryable(true)
///     .with_source(cause)
///     .into();
/// assert!(error.retryable());
/// let source = std::error::Error::source(&error).map(ToString::to_string);
/// assert_eq!(source.as_deref(), Some("connection reset"));
/// ```
#[derive(Debug)]
pub struct Failure {
    code: Code,
    message: String,
    retryable: bool,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
    backtrace: Backtrace,
}

impl Failure {
    /// A failure with `code` and `message`, not retryable and without a
    /// source. It captures a backtrace when `RUST_BACKTRACE` or
    /// `RUST_LIB_BACKTRACE` is set.
    #[must_use]
    pub fn new(code: impl ErrorCode, message: impl Into<String>) -> Failure {
        Failure {
            code: Code::of(code),
            message: message.into(),
            retryable: false,
            source: None,
            backtrace: Backtrace::capture(),
        }
    }

    /// This failure, retryable or not.
    #[must_use]
    pub fn with_retryable(mut self, retryable: bool) -> Failure {
        self.retryable = retryable;
        self
    }

    /// This failure with `source` as its cause, replacing any earlier one.
    /// Takes any standard error that may cross threads, or a string.
    #[must_use]
    pub fn with_source(
        mut self,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Failure {
        self.source = Some(source.into());
        self
    }

    /// The developer-facing explanation of the failure.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Failure {
    /// The message.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Failure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn std::error::Error + 'static))
    }
}

impl Fault for Failure {
    fn code(&self) -> Code {
        self.code
    }

    fn retryable(&self) -> bool {
        self.retryable
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        Some(&self.backtrace)
    }
}

/// Adds a stack frame to the error of a `Result` on its way out.
///
/// It works on any `Result` whose error converts into [`Error`]: an `Error`
/// or any [`Fault`].
pub trait Context<T> {
    /// The result, with `frame` as the error's new outermost operation.
    ///
    /// # Errors
    ///
    /// The error of `self`, converted into [`Error`], with the frame added.
    fn context(self, frame: impl fmt::Display) -> Result<T, Error>;

    /// [`Context::context`] with a frame built only when there is an
    /// error.
    ///
    /// # Errors
    ///
    /// The error of `self`, converted into [`Error`], with the frame added.
    fn with_context<D: fmt::Display>(self, frame: impl FnOnce() -> D) -> Result<T, Error>;
}

impl<T, E: Into<Error>> Context<T> for Result<T, E> {
    fn context(self, frame: impl fmt::Display) -> Result<T, Error> {
        self.map_err(|error| error.into().context(frame))
    }

    fn with_context<D: fmt::Display>(self, frame: impl FnOnce() -> D) -> Result<T, Error> {
        self.map_err(|error| error.into().context(frame()))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error as _;

    use super::{Context, Error, Failure};
    use crate::RuntimeCode;

    #[test]
    fn new_is_a_plain_failure() {
        let error = Error::new(RuntimeCode::Panicked, "boom");
        assert!(error.is(RuntimeCode::Panicked));
        assert_eq!(error.code_as::<RuntimeCode>(), Some(RuntimeCode::Panicked));
        assert!(!error.retryable());
        assert!(error.stack().is_empty());
        assert!(error.source().is_none());
        assert!(error.backtrace().is_some());
        assert_eq!(error.to_string(), "PANICKED: boom");
        let failure = error.downcast_ref::<Failure>().map(Failure::message);
        assert_eq!(failure, Some("boom"));
    }

    #[test]
    fn failure_keeps_retry_and_the_last_source() {
        let error: Error = Failure::new(RuntimeCode::Cancelled, "cannot read")
            .with_retryable(true)
            .with_source("first")
            .with_source(std::io::Error::other("second"))
            .into();
        assert!(error.retryable());
        let source = error.source().map(ToString::to_string);
        assert_eq!(source.as_deref(), Some("second"));
        assert_eq!(error.to_string(), "CANCELLED: cannot read");
    }

    #[test]
    fn context_prepends_the_outermost_frame() {
        let inner: Result<(), Failure> = Err(Failure::new(RuntimeCode::Cancelled, "gone"));
        let error = inner
            .context("read")
            .with_context(|| format!("load {}", 7))
            .context("")
            .unwrap_err();
        assert_eq!(error.stack(), ["load 7", "read"]);
        assert_eq!(error.to_string(), "load 7: read: CANCELLED: gone");
    }

    #[test]
    fn with_context_builds_no_frame_on_ok() {
        let ok: Result<u8, Error> = Ok(1);
        let value = ok.with_context(|| -> String { unreachable!("built on Ok") });
        assert_eq!(value.ok(), Some(1));
    }

    #[test]
    fn clone_shares_the_fault() {
        let error = Error::new(RuntimeCode::CannotBlock, "wait").context("call");
        let copy = error.clone();
        assert!(copy.is(RuntimeCode::CannotBlock));
        assert_eq!(copy.stack(), ["call"]);
        assert_eq!(copy.to_string(), error.to_string());
    }

    #[test]
    fn is_a_std_error() {
        let error: Box<dyn std::error::Error + Send + Sync> =
            Box::new(Error::new(RuntimeCode::Cancelled, "down"));
        assert_eq!(error.to_string(), "CANCELLED: down");
    }
}
