//! Error types and utilities.
error_chain! {
    errors {
        ScriptException(message: String) {
            description("JavaScript exception")
            display("JavaScript exception: {}", message)
        }
    }
}
