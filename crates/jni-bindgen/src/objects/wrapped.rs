use crate::conversion::class_convert::get_struct_mut;
use crate::objects::traits::FromJNI;
use jni::objects::JObject;
use jni::JNIEnv;
use std::ops::{Deref, DerefMut};

/// A wrapper for a Rust object that has been passed to Java
/// and is now passed back to Rust. This wrapper is used to
/// pass Java Lists containing Rust objects back to Rust.
pub struct Wrapped<'local, T> {
    inner: &'local mut T,
}

impl<'local, T> Wrapped<'local, T> {
    /// Create a new wrapper.
    /// This will create a new wrapper for the given Rust object.
    /// The wrapper will be valid for the lifetime of the given
    /// JNI environment.
    pub fn new(inner: &'local mut T) -> Self {
        Self { inner }
    }
}

impl<'local, T> Deref for Wrapped<'local, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'local, T> DerefMut for Wrapped<'local, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

impl<'local, T: 'static> FromJNI<'local> for Wrapped<'local, T> {
    fn from_jni(env: &mut JNIEnv<'local>, obj: JObject) -> crate::Result<Self> {
        let inner = get_struct_mut::<T>(env, obj)?;

        Ok(Wrapped::new(inner))
    }
}

impl<'local, T: 'static> FromJNI<'local> for Option<Wrapped<'local, T>> {
    fn from_jni(env: &mut JNIEnv<'local>, obj: JObject) -> crate::Result<Self> {
        if obj.is_null() {
            Ok(None)
        } else {
            Ok(Some(Wrapped::from_jni(env, obj)?))
        }
    }
}
