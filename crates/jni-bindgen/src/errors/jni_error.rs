use jni::errors::Error as JNIErrorEnum;
use jni::JNIEnv;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use strum_macros::Display;

/// The (java) class of an error.
/// This is used to determine which exception to throw.
/// If the class is not known, the default is `java/lang/RuntimeException`.
#[derive(Debug, Display, Default, Clone)]
pub enum ErrorClass {
    #[strum(serialize = "java/lang/NullPointerException")]
    NullPointer,
    #[strum(serialize = "java/lang/IllegalArgumentException")]
    IllegalArgument,
    #[strum(serialize = "java/lang/IllegalStateException")]
    IllegalState,
    #[strum(serialize = "java/lang/UnsupportedOperationException")]
    UnsupportedOperation,
    #[default]
    #[strum(serialize = "java/lang/RuntimeException")]
    Runtime,
    #[strum(serialize = "java/lang/Exception")]
    Exception,
    #[strum(serialize = "java/lang/NoSuchMethodException")]
    NoSuchMethod,
    #[strum(serialize = "java/lang/NoSuchFieldException")]
    NoSuchField,
    #[strum(serialize = "com/github/markusjx/jnibindgen/NativeExecutionException")]
    NativeExecutionException,
    /// Any other class.
    /// This is used to throw a custom exception.
    /// The string is the class name.
    /// The class name must be in the format `java.lang.String`
    /// or `java/lang/String`.
    Any(String),
}

impl From<JNIErrorEnum> for ErrorClass {
    fn from(value: JNIErrorEnum) -> Self {
        match value {
            JNIErrorEnum::MethodNotFound { .. }
            | JNIErrorEnum::JavaVMMethodNotFound(..)
            | JNIErrorEnum::JNIEnvMethodNotFound(..) => ErrorClass::NoSuchMethod,
            JNIErrorEnum::NullPtr(..) | JNIErrorEnum::NullDeref(..) => ErrorClass::NullPointer,
            JNIErrorEnum::FieldNotFound { .. } => ErrorClass::NoSuchField,
            JNIErrorEnum::InvalidArgList(..) => ErrorClass::IllegalArgument,
            _ => ErrorClass::Runtime,
        }
    }
}

impl ErrorClass {
    /// Converts the class name into its JNI representation.
    pub fn into_class_name(self) -> String {
        match self {
            ErrorClass::Any(s) => s.replace('.', "/"),
            e => e.to_string(),
        }
    }
}

/// An error that can be thrown in java.
/// This is used to throw exceptions from rust.
/// If the class is not known, the default is `java.lang.RuntimeException`.
#[derive(Debug)]
pub struct JNIError {
    /// The error message.
    pub message: String,
    /// The class of the error.
    pub class: Option<ErrorClass>,
}

impl JNIError {
    /// Create a new JNI error.
    /// The class is optional and defaults to `java.lang.RuntimeException`.
    ///
    /// You may want to use the [`crate::bail`] or [`crate::error`] macros instead.
    ///
    /// # Arguments
    /// * `message` - The error message.
    /// * `class` - The class of the error.
    ///
    /// # Returns
    /// The new JNI error.
    ///
    /// # Example
    /// ```
    /// use jni_bindgen::errors::jni_error::{JNIError, ErrorClass};
    ///
    /// fn throw_error() -> jni_bindgen::Result<()> {
    ///     Err(JNIError::new("Error", None))
    /// }
    ///
    /// fn throw_custom_error() -> jni_bindgen::Result<()> {
    ///     Err(JNIError::new(
    ///         "Error",
    ///         Some(ErrorClass::Any(
    ///             "com/example/MyException".to_string()
    ///         ))
    ///     ))
    /// }
    /// ```
    pub fn new<T: ToString>(message: T, class: Option<ErrorClass>) -> Self {
        JNIError {
            message: message.to_string(),
            class,
        }
    }

    /// Create a new JNI error with the given message.
    /// The class of the error is `java.lang.RuntimeException`.
    /// This is a shortcut for `JNIError::new(message, None)`.
    pub fn runtime_error<T: ToString>(message: T) -> Self {
        JNIError::new(message, None)
    }

    /// Throw the error in java.
    /// This will throw the error as an exception in java.
    ///
    /// # Example
    /// ```
    /// use std::thread::yield_now;
    /// use jni_bindgen::errors::jni_error::{JNIError, ErrorClass};
    /// use jni_bindgen::jni;
    ///
    /// fn throw_custom_error() -> jni_bindgen::Result<()> {
    ///     Err(JNIError::new(
    ///         "Error",
    ///         Some(ErrorClass::Any(
    ///             "com/example/MyException".to_string()
    ///         ))
    ///     ))
    /// }
    ///
    /// #[no_mangle]
    /// pub extern "system" fn Java_com_example_MyClass_throwError<'local>(
    ///    mut env: jni::JNIEnv<'local>,
    ///    _: jni::objects::JClass<'local>,
    /// ) {
    ///    if let Err(e) = throw_custom_error() {
    ///       e.throw(&mut env);
    ///    }
    /// }
    /// ```
    pub fn throw(self, env: &mut JNIEnv) {
        let _ = env.throw_new(
            self.class.unwrap_or_default().into_class_name(),
            self.message,
        );
    }

    /// Set the class of the error if it is not already set.
    pub fn or_class(mut self, class: ErrorClass) -> Self {
        if self.class.is_none() {
            self.class = Some(class);
        }

        self
    }
}

impl Display for JNIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for JNIError {}

impl From<anyhow::Error> for JNIError {
    fn from(value: anyhow::Error) -> Self {
        JNIError::new(value, None)
    }
}

impl From<JNIErrorEnum> for JNIError {
    fn from(value: JNIErrorEnum) -> Self {
        JNIError::new(value.to_string(), Some(value.into()))
    }
}

impl From<String> for JNIError {
    fn from(value: String) -> Self {
        JNIError::new(value, None)
    }
}

impl From<&str> for JNIError {
    fn from(value: &str) -> Self {
        JNIError::new(value, None)
    }
}
