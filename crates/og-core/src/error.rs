//! Errors as values: every fallible engine call returns `Result<_, Error>`.
//!
//! An [`Error`] carries a coarse [`ErrorKind`] a caller can branch on and a
//! reason string written for the person (or agent) reading it. The probe and
//! the TypeScript API surface it as `{ ok: false, reason }`.

/// What went wrong, coarsely; the reason says exactly what.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// A handle or entity whose slot has been freed (or never existed).
    DeadHandle,
    /// A name, type, resource, component or field that is not there.
    NotFound,
    /// Text or bytes that do not follow their format (`.ogt`, snapshot, JSON).
    Parse,
    /// A schema that is invalid, or two layouts that do not match.
    Schema,
    /// An access a system did not declare (read or write).
    Access,
    /// An argument outside what the call accepts.
    Invalid,
}

/// An error value: a kind plus a human-readable reason.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{reason}")]
pub struct Error {
    kind: ErrorKind,
    reason: String,
}

impl Error {
    pub fn new(kind: ErrorKind, reason: impl Into<String>) -> Self {
        Self {
            kind,
            reason: reason.into(),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// The same error with `context` prepended to its reason (`"context: reason"`).
    pub fn context(self, context: impl std::fmt::Display) -> Self {
        Self {
            kind: self.kind,
            reason: format!("{context}: {}", self.reason),
        }
    }
}

/// `Result` with the engine's [`Error`].
pub type Result<T> = core::result::Result<T, Error>;
