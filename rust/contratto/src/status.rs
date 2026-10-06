//! The error every generated method returns.

use std::fmt;

use buffa_types::google::protobuf::Any;

use crate::Code;

/// The failure of a call: a mirror of `google.rpc.Status`.
///
/// `code` classifies the failure, `message` explains it to a developer, and
/// `details` carries machine-readable payloads (`google.rpc.BadRequest`,
/// `google.rpc.ErrorInfo`, or any other message packed into `Any`).
///
/// A component that never leaves its process uses `Status` without
/// depending on an RPC stack.
///
/// ```
/// use contratto::{Code, Status};
///
/// let status = Status::not_found("no greeting for `ada`");
/// assert_eq!(status.code(), Code::NotFound);
/// assert_eq!(status.to_string(), "NOT_FOUND: no greeting for `ada`");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    code: Code,
    message: String,
    details: Vec<Any>,
}

macro_rules! constructors {
    ($($(#[doc = $doc:literal])* $name:ident => $code:ident;)*) => {
        $(
            $(#[doc = $doc])*
            #[must_use]
            pub fn $name(message: impl Into<String>) -> Status {
                Status::new(Code::$code, message)
            }
        )*
    };
}

impl Status {
    /// A status with `code` and `message` and no details.
    #[must_use]
    pub fn new(code: Code, message: impl Into<String>) -> Status {
        Status {
            code,
            message: message.into(),
            details: Vec::new(),
        }
    }

    constructors! {
        /// A `CANCELLED` status: the operation was cancelled.
        cancelled => Cancelled;
        /// An `UNKNOWN` status: an error that fits no other code.
        unknown => Unknown;
        /// An `INVALID_ARGUMENT` status: the request is invalid.
        invalid_argument => InvalidArgument;
        /// A `DEADLINE_EXCEEDED` status: the deadline expired first.
        deadline_exceeded => DeadlineExceeded;
        /// A `NOT_FOUND` status: a requested entity was not found.
        not_found => NotFound;
        /// An `ALREADY_EXISTS` status: the entity already exists.
        already_exists => AlreadyExists;
        /// A `PERMISSION_DENIED` status: the caller may not do this.
        permission_denied => PermissionDenied;
        /// A `RESOURCE_EXHAUSTED` status: a quota or resource ran out.
        resource_exhausted => ResourceExhausted;
        /// A `FAILED_PRECONDITION` status: the system is not in the state
        /// the operation requires.
        failed_precondition => FailedPrecondition;
        /// An `ABORTED` status: aborted, typically by a concurrency conflict.
        aborted => Aborted;
        /// An `OUT_OF_RANGE` status: attempted past the valid range.
        out_of_range => OutOfRange;
        /// An `UNIMPLEMENTED` status: the operation is not supported.
        unimplemented => Unimplemented;
        /// An `INTERNAL` status: an invariant of the system is broken.
        internal => Internal;
        /// An `UNAVAILABLE` status: the service is unavailable for now.
        unavailable => Unavailable;
        /// A `DATA_LOSS` status: unrecoverable data loss or corruption.
        data_loss => DataLoss;
        /// An `UNAUTHENTICATED` status: the request has no valid
        /// credentials.
        unauthenticated => Unauthenticated;
    }

    /// This status with `details` appended to the details it already holds.
    #[must_use]
    pub fn with_details(mut self, details: impl IntoIterator<Item = Any>) -> Status {
        self.details.extend(details);
        self
    }

    /// Class of the failure, one of the `google.rpc.Code` values.
    #[must_use]
    pub fn code(&self) -> Code {
        self.code
    }

    /// The developer-facing explanation of the failure.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The machine-readable payloads of the failure, in order.
    #[must_use]
    pub fn details(&self) -> &[Any] {
        &self.details
    }

    /// The code, message, and details, moved out of the status.
    #[must_use]
    pub fn into_parts(self) -> (Code, String, Vec<Any>) {
        (self.code, self.message, self.details)
    }
}

impl fmt::Display for Status {
    /// `CODE: message`, or the code alone when the message is empty.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.message.is_empty() {
            write!(f, "{}", self.code)
        } else {
            write!(f, "{}: {}", self.code, self.message)
        }
    }
}

impl std::error::Error for Status {}

#[cfg(test)]
mod tests {
    use buffa_types::google::protobuf::Any;

    use super::Status;
    use crate::Code;

    #[test]
    fn constructors_set_their_code() {
        let cases = [
            (Status::cancelled("m"), Code::Cancelled),
            (Status::unknown("m"), Code::Unknown),
            (Status::invalid_argument("m"), Code::InvalidArgument),
            (Status::deadline_exceeded("m"), Code::DeadlineExceeded),
            (Status::not_found("m"), Code::NotFound),
            (Status::already_exists("m"), Code::AlreadyExists),
            (Status::permission_denied("m"), Code::PermissionDenied),
            (Status::resource_exhausted("m"), Code::ResourceExhausted),
            (Status::failed_precondition("m"), Code::FailedPrecondition),
            (Status::aborted("m"), Code::Aborted),
            (Status::out_of_range("m"), Code::OutOfRange),
            (Status::unimplemented("m"), Code::Unimplemented),
            (Status::internal("m"), Code::Internal),
            (Status::unavailable("m"), Code::Unavailable),
            (Status::data_loss("m"), Code::DataLoss),
            (Status::unauthenticated("m"), Code::Unauthenticated),
        ];
        for (status, code) in cases {
            assert_eq!(status.code(), code);
            assert_eq!(status.message(), "m");
            assert!(status.details().is_empty());
        }
    }

    #[test]
    fn details_are_kept_in_order() {
        let first = Any {
            type_url: "type.googleapis.com/google.rpc.ErrorInfo".into(),
            ..Any::default()
        };
        let second = Any {
            type_url: "type.googleapis.com/google.rpc.BadRequest".into(),
            ..Any::default()
        };
        let status = Status::invalid_argument("bad name")
            .with_details([first.clone()])
            .with_details([second.clone()]);
        assert_eq!(status.details(), [first.clone(), second.clone()]);

        let (code, message, details) = status.into_parts();
        assert_eq!(code, Code::InvalidArgument);
        assert_eq!(message, "bad name");
        assert_eq!(details, [first, second]);
    }

    #[test]
    fn display_shows_code_and_message() {
        assert_eq!(Status::internal("boom").to_string(), "INTERNAL: boom");
        assert_eq!(Status::new(Code::Aborted, "").to_string(), "ABORTED");
    }

    #[test]
    fn is_a_std_error() {
        let error: Box<dyn std::error::Error + Send + Sync> = Box::new(Status::unavailable("down"));
        assert_eq!(error.to_string(), "UNAVAILABLE: down");
    }
}
