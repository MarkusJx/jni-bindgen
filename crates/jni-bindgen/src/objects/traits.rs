use jni::objects::JObject;
use jni::JNIEnv;

/// Convert a Java object into a Rust object.
pub trait FromJNI<'local>: Sized {
    /// Convert a Java object into a Rust object.
    /// This will convert the given Java object into a Rust object.
    /// If the Java object is null and `self` is not an [`Option`],
    /// this will return an error.
    ///
    /// # Arguments
    /// * `env` - The JNI environment.
    /// * `obj` - The Java object.
    ///
    /// # Returns
    /// The Rust object.
    fn from_jni(env: &mut JNIEnv<'local>, obj: JObject<'local>) -> crate::Result<Self>;
}

/// Convert a previously into Java converted object back into a Rust object.
pub trait ObjectFromJNI<'local>: FromJNI<'local> + Send + Sync {}

/// Convert a Rust object into a Java object.
pub trait IntoJNI {
    /// Convert a Rust object into a Java object.
    /// This will convert the given Rust object into a Java object.
    /// The Java object may be null if `self` is an [`Option`].
    /// Otherwise, the Java object must not be null.
    ///
    /// # Arguments
    /// * `env` - The JNI environment.
    ///
    /// # Returns
    /// The Java object.
    fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> crate::Result<JObject<'a>>;
}

/// Convert any [`Result`] into a JNI result.
pub trait IntoJNIResult<T> {
    fn into_jni_result(self) -> crate::Result<T>;
}
