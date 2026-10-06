//! The canonical error codes of `google.rpc.Code`.

use std::fmt;

macro_rules! codes {
    ($($(#[doc = $doc:literal])* $variant:ident = $value:literal, $name:literal;)*) => {
        /// Class of a failure: one of the 17 `google.rpc.Code` values, the
        /// same set gRPC and Connect use.
        ///
        /// The numeric value of each variant is its `google.rpc.Code` number.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[repr(i32)]
        pub enum Code {
            $($(#[doc = $doc])* $variant = $value,)*
        }

        impl Code {
            /// Every code, in `google.rpc.Code` order.
            pub const ALL: [Code; 17] = [$(Code::$variant,)*];

            /// The `google.rpc.Code` number of this code.
            #[must_use]
            pub const fn as_i32(self) -> i32 {
                self as i32
            }

            /// The code with the given `google.rpc.Code` number, or `None`
            /// for a number outside the 17 values.
            #[must_use]
            pub const fn from_i32(value: i32) -> Option<Code> {
                match value {
                    $($value => Some(Code::$variant),)*
                    _ => None,
                }
            }

            /// The `google.rpc.Code` name of this code, such as
            /// `INVALID_ARGUMENT`.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Code::$variant => $name,)*
                }
            }
        }
    };
}

codes! {
    /// Not an error; returned on success.
    Ok = 0, "OK";
    /// The operation was cancelled, typically by the caller.
    Cancelled = 1, "CANCELLED";
    /// An error that fits no other code.
    Unknown = 2, "UNKNOWN";
    /// The caller specified an invalid argument, whatever the state of the
    /// system.
    InvalidArgument = 3, "INVALID_ARGUMENT";
    /// The deadline expired before the operation could complete.
    DeadlineExceeded = 4, "DEADLINE_EXCEEDED";
    /// A requested entity was not found.
    NotFound = 5, "NOT_FOUND";
    /// The entity the caller attempted to create already exists.
    AlreadyExists = 6, "ALREADY_EXISTS";
    /// The caller is known but may not perform the operation.
    PermissionDenied = 7, "PERMISSION_DENIED";
    /// A resource, such as a quota or disk space, has been exhausted.
    ResourceExhausted = 8, "RESOURCE_EXHAUSTED";
    /// The system is not in the state the operation requires.
    FailedPrecondition = 9, "FAILED_PRECONDITION";
    /// The operation was aborted, typically by a concurrency conflict.
    Aborted = 10, "ABORTED";
    /// The operation was attempted past the valid range.
    OutOfRange = 11, "OUT_OF_RANGE";
    /// The operation is not implemented or not supported.
    Unimplemented = 12, "UNIMPLEMENTED";
    /// An invariant the system relies on is broken.
    Internal = 13, "INTERNAL";
    /// The service is currently unavailable; a retry may succeed.
    Unavailable = 14, "UNAVAILABLE";
    /// Unrecoverable data loss or corruption.
    DataLoss = 15, "DATA_LOSS";
    /// The request has no valid credentials for the operation.
    Unauthenticated = 16, "UNAUTHENTICATED";
}

impl From<Code> for i32 {
    fn from(code: Code) -> i32 {
        code.as_i32()
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::Code;

    #[test]
    fn numbers_round_trip() {
        for (index, code) in Code::ALL.into_iter().enumerate() {
            assert_eq!(code.as_i32(), i32::try_from(index).unwrap());
            assert_eq!(Code::from_i32(code.as_i32()), Some(code));
        }
        assert_eq!(Code::from_i32(-1), None);
        assert_eq!(Code::from_i32(17), None);
    }

    #[test]
    fn names_follow_google_rpc() {
        assert_eq!(Code::Cancelled.as_str(), "CANCELLED");
        assert_eq!(Code::InvalidArgument.to_string(), "INVALID_ARGUMENT");
        assert_eq!(i32::from(Code::Unauthenticated), 16);
    }
}
