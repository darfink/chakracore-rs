//! Error types and utilities.
error_chain! {
    errors {
        /// A variant indicating that a runtime error has occured.
        ScriptException(message: String) {
            description("JavaScript exception")
            display("JavaScript exception: {}", message)
        }
        /// An error caused by incorrect code syntax.
        ScriptCompile(message: String) {
            description("JavaScript compile error")
            display("JavaScript compile error: {}", message)
        }
    }
}
