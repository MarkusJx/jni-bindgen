use crate::bail_class;
use crate::errors::jni_error::ErrorClass;
use jni::objects::JObject;
use jni::sys::jlong;
use jni::JNIEnv;
use std::any::TypeId;
use std::hash::{Hash, Hasher};

fn hash_type<T: 'static>() -> jlong {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    TypeId::of::<T>().hash(&mut hasher);
    hasher.finish() as jlong
}

fn check_ptr<T: 'static>(env: &mut JNIEnv, obj: &JObject, ptr: *const T) -> crate::Result<()> {
    if ptr.is_null() {
        bail_class!(ErrorClass::NullPointer, "The pointer is null");
    }

    let cls = env.get_object_class(obj)?;
    let type_id = env
        .call_static_method(cls, "getTypeHash", "()J", &[])?
        .j()?;

    if type_id != hash_type::<T>() {
        bail_class!(
            ErrorClass::IllegalArgument,
            "Expected object of type {}",
            std::any::type_name::<T>()
        );
    }

    Ok(())
}

/// Get a pointer to a Rust struct from a Java object.
pub fn get_struct<'a, T: 'static>(env: &mut JNIEnv<'a>, obj: JObject) -> crate::Result<&'a T> {
    let ptr = env.call_method(&obj, "getPtr", "()J", &[])?.j()? as *const T;
    check_ptr(env, &obj, ptr)?;

    unsafe { Ok(&*ptr) }
}

/// Get a mutable pointer to a Rust struct from a Java object.
pub fn get_struct_mut<'a, T: 'static>(
    env: &mut JNIEnv<'a>,
    obj: JObject,
) -> crate::Result<&'a mut T> {
    let ptr = env.call_method(&obj, "getPtr", "()J", &[])?.j()? as *mut T;
    check_ptr(env, &obj, ptr)?;

    unsafe { Ok(&mut *ptr) }
}
