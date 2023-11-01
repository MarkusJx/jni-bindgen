use crate::errors::jni_error::JNIError;
use crate::objects::traits::IntoJNIResult;

/// The result type used by jni-bindgen.
/// This is a shortcut for `std::result::Result<T, JNIError>`.
/// This is used by all functions in jni-bindgen.
/// You can use the [`IntoJNIResult`] trait to convert a
/// [`std::result::Result`] to a [`Result`].
pub type Result<T> = std::result::Result<T, JNIError>;

impl<T, R: Into<JNIError>> IntoJNIResult<T> for std::result::Result<T, R> {
    fn into_jni_result(self) -> Result<T> {
        self.map_err(|e| e.into())
    }
}

/// Return from the current function with an error.
/// This will return an error with the given message.
/// This is a shortcut for `return Err(JNIError::new(message, None));`.
///
/// The error will be of type `java.lang.RuntimeException`.
/// If you want to throw a custom exception, use [`bail_class!`] instead.
///
/// # Example
/// ```
/// use jni_bindgen::bail;
///
/// fn throw_error() -> jni_bindgen::Result<()> {
///    bail!("Error");
/// }
/// ```
#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::errors::jni_error::JNIError::new(
            format!($($arg)*),
            None,
        ));
    };
}

/// Return from the current function with an error.
/// This will return an error with the given message and class.
/// This is a shortcut for `return Err(JNIError::new(message, Some(class)));`.
///
/// # Example
/// ```
/// use jni_bindgen::bail_class;
/// use jni_bindgen::errors::jni_error::ErrorClass;
///
/// fn throw_custom_error() -> jni_bindgen::Result<()> {
///     bail_class!(
///         ErrorClass::Any("com/example/MyException".to_string()),
///         "Error"
///     );
/// }
/// ```
#[macro_export]
macro_rules! bail_class {
    ($cls: expr, $($arg:tt)*) => {
        return Err($crate::errors::jni_error::JNIError::new(
            format!($($arg)*),
            Some($cls),
        ));
    };
}

/// Create a new error.
/// This will create a new error with the given message.
/// This is a shortcut for `JNIError::new(message, None)`.
///
/// The error will be of type `java.lang.RuntimeException`.
/// If you want to throw a custom exception, use [`error_class!`] instead.
///
/// # Example
/// ```
/// use jni_bindgen::error;
///
/// fn throw_error() -> jni_bindgen::Result<()> {
///     Err(error!("Error"))
/// }
/// ```
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::errors::jni_error::JNIError::new(
            format!($($arg)*),
            None,
        )
    };
}

/// Create a new error.
/// This will create a new error with the given message and class.
/// This is a shortcut for `JNIError::new(message, Some(class))`.
///
/// # Example
/// ```
/// use jni_bindgen::error_class;
/// use jni_bindgen::errors::jni_error::ErrorClass;
///
/// fn throw_custom_error() -> jni_bindgen::Result<()> {
///    Err(error_class!(
///       ErrorClass::Any("com/example/MyException".to_string()),
///       "Error"
///    ))
/// }
/// ```
#[macro_export]
macro_rules! error_class {
    ($cls: expr, $($arg:tt)*) => {
        $crate::errors::jni_error::JNIError::new(
            format!($($arg)*),
            Some($cls),
        )
    };
}
