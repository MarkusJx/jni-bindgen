use crate::conversion::traits::FromJNI;
use jni::objects::{JObject, JString};
use jni::JNIEnv;
use std::ops::{Deref, DerefMut};

pub struct Wrapped<'local, T> {
    inner: *mut T,
    lifetime: std::marker::PhantomData<&'local ()>,
}

impl<'local, T> Wrapped<'local, T> {
    pub fn new(inner: *mut T) -> Self {
        Self {
            inner,
            lifetime: Default::default(),
        }
    }
}

impl<'local, T> Deref for Wrapped<'local, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

impl<'local, T> DerefMut for Wrapped<'local, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner }
    }
}

impl<'local, T: 'static> FromJNI<'local> for Wrapped<'local, T> {
    fn from_jni(env: &mut JNIEnv<'local>, obj: JObject) -> anyhow::Result<Self> {
        let ptr = env.call_method(&obj, "getPtr", "()J", &[])?.j()? as *mut T;
        let cls = env.get_object_class(&obj)?;
        let type_name_obj = JString::from(
            env.call_static_method(cls, "getTypeName", "()Ljava/lang/String;", &[])?
                .l()?,
        );
        let type_name: String = env.get_string(&type_name_obj)?.into();

        if type_name != std::any::type_name::<T>() {
            anyhow::bail!(
                "Expected object of type {}, but got {type_name}",
                std::any::type_name::<T>()
            );
        }

        Ok(Wrapped::new(ptr))
    }
}

impl<'local, T: 'static> FromJNI<'local> for Option<Wrapped<'local, T>> {
    fn from_jni(env: &mut JNIEnv<'local>, obj: JObject) -> anyhow::Result<Self> {
        if obj.is_null() {
            Ok(None)
        } else {
            Ok(Some(Wrapped::from_jni(env, obj)?))
        }
    }
}
