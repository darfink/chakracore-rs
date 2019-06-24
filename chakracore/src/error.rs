use chakracore_sys::JsErrorCode;
use std::error::Error as StdError;
use std::fmt;

/// The result of a detour operation.
pub type Result<T> = ::std::result::Result<T, Error>;

/// A representation of all possible errors.
#[derive(Debug)]
pub enum Error {
  /// A variant indicating that a runtime error has occured.
  ScriptException(String),
  /// An error caused by incorrect code syntax.
  ScriptCompilation(String),
  /// A JSRT call failed.
  JsrtCall(JsErrorCode),
}

impl StdError for Error {}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::ScriptException(message) => write!(f, "JavaScript exception: {}", message),
      Error::ScriptCompilation(message) => write!(f, "JavaScript compile error: {}", message),
      Error::JsrtCall(error) => write!(f, "JSRT call error: {:?}", error),
    }
  }
}
