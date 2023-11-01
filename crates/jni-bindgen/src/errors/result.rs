use crate::errors::jni_error::JNIError;
use crate::objects::traits::IntoJNIResult;

pub type Result<T> = std::result::Result<T, JNIError>;

impl<T, R: Into<JNIError>> IntoJNIResult<T> for std::result::Result<T, R> {
    fn into_jni_result(self) -> Result<T> {
        self.map_err(|e| e.into())
    }
}

#[macro_export]
macro_rules! bail {
    ($cls: expr, $($arg:tt)*) => {
        return Err($crate::errors::jni_error::JNIError::new(
                format!($($arg)*),
                Some($cls),
        ));
    };
    ($($arg:tt)*) => {
        return Err($crate::errors::jni_error::JNIError::new(
                format!($($arg)*),
                None,
        ));
    };
}

#[macro_export]
macro_rules! error {
    ($cls: expr, $($arg:tt)*) => {
        $crate::errors::jni_error::JNIError::new(
                format!($($arg)*),
                Some($cls),
        )
    };
    ($($arg:tt)*) => {
        $crate::errors::jni_error::JNIError::new(
                format!($($arg)*),
                None,
        )
    };
}
