use crate::objects::wrapped::Wrapped;
use jni::objects::JObject;
use jni::JNIEnv;

pub trait FromJObject: Sized {
    fn from_jobject<'local>(
        env: &mut JNIEnv<'local>,
        obj: JObject,
    ) -> crate::Result<Wrapped<'local, Self>>;
}

pub trait FromJNI<'local>: Sized {
    fn from_jni(env: &mut JNIEnv<'local>, obj: JObject) -> crate::Result<Self>;
}

pub trait IntoJNI {
    fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> crate::Result<JObject<'a>>;
}

pub trait IntoJNIResult<T> {
    fn into_jni_result(self) -> crate::Result<T>;
}
