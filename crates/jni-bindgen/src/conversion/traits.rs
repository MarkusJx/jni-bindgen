use crate::objects::wrapped::Wrapped;
use jni::objects::JObject;
use jni::JNIEnv;

pub trait FromJObject: Sized {
    fn from_jobject<'local>(
        env: &mut JNIEnv<'local>,
        obj: JObject,
    ) -> anyhow::Result<Wrapped<'local, Self>>;
}

pub trait FromJNI<'local>: Sized {
    fn from_jni(env: &mut JNIEnv<'local>, obj: JObject) -> anyhow::Result<Self>;
}

pub trait IntoJNI {
    fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> anyhow::Result<JObject<'a>>;
}
