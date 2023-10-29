use anyhow::bail;
use jni::objects::JObject;
use jni::JNIEnv;

pub fn get_struct<'a, T>(env: &mut JNIEnv, obj: JObject<'a>) -> anyhow::Result<&'a T> {
    let ptr = env.call_method(obj, "getPtr", "()J", &[])?.j()? as *const T;
    if ptr.is_null() {
        bail!("The pointer is null");
    }

    unsafe { Ok(&*ptr) }
}

pub fn get_struct_mut<'a, T>(env: &mut JNIEnv, obj: JObject<'a>) -> anyhow::Result<&'a mut T> {
    let ptr = env.call_method(obj, "getPtr", "()J", &[])?.j()? as *mut T;
    if ptr.is_null() {
        bail!("The pointer is null");
    }

    unsafe { Ok(&mut *ptr) }
}
