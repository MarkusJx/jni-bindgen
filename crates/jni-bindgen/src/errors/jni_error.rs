use jni::errors::Error as JNIErrorEnum;
use jni::JNIEnv;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use strum_macros::Display;

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
    pub fn into_error_class(self) -> String {
        match self {
            ErrorClass::Any(s) => s,
            _ => self.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct JNIError {
    pub message: String,
    pub class: Option<ErrorClass>,
}

impl JNIError {
    pub fn new<T: ToString>(message: T, class: Option<ErrorClass>) -> Self {
        JNIError {
            message: message.to_string(),
            class,
        }
    }

    pub fn runtime_error<T: ToString>(message: T) -> Self {
        JNIError::new(message, None)
    }

    pub fn throw(self, env: &mut JNIEnv) {
        println!("Throwing error: {}", self.class.clone().unwrap_or_default());
        let _ = env.throw_new(
            self.class.unwrap_or_default().into_error_class(),
            self.message,
        );
    }

    pub fn or_class(mut self, class: ErrorClass) -> Self {
        println!("Setting class to {:?} from {:?}", class, self);
        if self.class.is_none() {
            println!("Setting class to {:?}", class);
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
