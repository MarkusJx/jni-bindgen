use crate::conversion::option_convert::*;
use jni::objects::{JObject, JString};
use jni::sys::jobject;
use jni::JNIEnv;
use std::collections::HashMap;
use std::hash::Hash;

pub trait Convert: Sized {
    fn from_jni(env: &mut JNIEnv, obj: JObject) -> anyhow::Result<Self>;

    fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> anyhow::Result<JObject<'a>>;
}

macro_rules! impl_convert {
    ($ty: ty, $from: ident, $to: ident) => {
        impl Convert for $ty {
            fn from_jni(env: &mut JNIEnv, obj: JObject) -> anyhow::Result<Self> {
                $from(env, obj)?.ok_or(anyhow::anyhow!("The value is null"))
            }

            fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> anyhow::Result<JObject<'a>> {
                Ok(unsafe { JObject::from_raw($to(env, Some(self))?) })
            }
        }

        impl Convert for Option<$ty> {
            fn from_jni(env: &mut JNIEnv, obj: JObject) -> anyhow::Result<Self> {
                $from(env, obj)
            }

            fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> anyhow::Result<JObject<'a>> {
                Ok(unsafe { JObject::from_raw($to(env, self)?) })
            }
        }
    };
}

impl_convert!(String, string_from_jni, string_into_jni);
impl_convert!(i32, i32_from_jni, i32_into_jni);
impl_convert!(i64, i64_from_jni, i64_into_jni);
impl_convert!(f32, f32_from_jni, f32_into_jni);
impl_convert!(f64, f64_from_jni, f64_into_jni);
impl_convert!(bool, bool_from_jni, bool_into_jni);
impl_convert!(i16, i16_from_jni, i16_into_jni);
impl_convert!(i8, i8_from_jni, i8_into_jni);
impl_convert!(u16, u16_from_jni, u16_into_jni);

impl Convert for JObject<'_> {
    fn from_jni(_: &mut JNIEnv, obj: JObject) -> anyhow::Result<Self> {
        Ok(unsafe { JObject::from_raw(*obj) })
    }

    fn into_jni<'a>(self, _: &mut JNIEnv<'a>) -> anyhow::Result<JObject<'a>> {
        Ok(unsafe { JObject::from_raw(*self) })
    }
}

impl Convert for JString<'_> {
    fn from_jni(_: &mut JNIEnv, obj: JObject) -> anyhow::Result<Self> {
        Ok(unsafe { JString::from(JObject::from_raw(*obj)) })
    }

    fn into_jni<'a>(self, _: &mut JNIEnv<'a>) -> anyhow::Result<JObject<'a>> {
        Ok(unsafe { JObject::from_raw(self.as_raw()) })
    }
}

impl<K: Convert + Eq + Hash, V: Convert> Convert for HashMap<K, V> {
    fn from_jni(env: &mut JNIEnv, obj: JObject) -> anyhow::Result<Self> {
        into_hashmap(env, obj)
    }

    fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> anyhow::Result<JObject<'a>> {
        Ok(unsafe { JObject::from_raw(from_hashmap(env, self)?) })
    }
}

impl<K: Convert + Eq + Hash, V: Convert> Convert for Option<HashMap<K, V>> {
    fn from_jni(env: &mut JNIEnv, obj: JObject) -> anyhow::Result<Self> {
        match obj.is_null() {
            true => Ok(None),
            false => Ok(Some(into_hashmap(env, obj)?)),
        }
    }

    fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> anyhow::Result<JObject<'a>> {
        match self {
            Some(map) => Ok(unsafe { JObject::from_raw(from_hashmap(env, map)?) }),
            None => Ok(JObject::null()),
        }
    }
}

impl<T: Convert> Convert for Vec<T> {
    fn from_jni(env: &mut JNIEnv, obj: JObject) -> anyhow::Result<Self> {
        into_vec(env, obj)
    }

    fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> anyhow::Result<JObject<'a>> {
        Ok(unsafe { JObject::from_raw(from_vec(env, self)?) })
    }
}

impl<T: Convert> Convert for Option<Vec<T>> {
    fn from_jni(env: &mut JNIEnv, obj: JObject) -> anyhow::Result<Self> {
        match obj.is_null() {
            true => Ok(None),
            false => Ok(Some(into_vec(env, obj)?)),
        }
    }

    fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> anyhow::Result<JObject<'a>> {
        match self {
            Some(vec) => Ok(unsafe { JObject::from_raw(from_vec(env, vec)?) }),
            None => Ok(JObject::null()),
        }
    }
}

pub fn into_hashmap<K: Convert + Eq + Hash, V: Convert>(
    env: &mut JNIEnv,
    obj: JObject,
) -> anyhow::Result<HashMap<K, V>> {
    let j_map = env.get_map(&obj)?;
    let mut iter = j_map.iter(env)?;
    let mut map = HashMap::new();

    while let Some((k, v)) = iter.next(env)? {
        map.insert(K::from_jni(env, k)?, V::from_jni(env, v)?);
    }

    Ok(map)
}

pub fn from_hashmap<K: Convert + Eq + Hash, V: Convert>(
    env: &mut JNIEnv,
    map: HashMap<K, V>,
) -> anyhow::Result<jobject> {
    let j_map = env.new_object("java/util/HashMap", "()V", &[])?;

    for (k, v) in map {
        let key = k.into_jni(env)?;
        let value = v.into_jni(env)?;
        env.call_method(
            &j_map,
            "put",
            "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
            &[key.as_ref().into(), value.as_ref().into()],
        )?;
    }

    Ok(j_map.into_raw())
}

pub fn into_vec<T: Convert>(env: &mut JNIEnv, obj: JObject) -> anyhow::Result<Vec<T>> {
    let j_vec = env.get_list(&obj)?;
    let mut iter = j_vec.iter(env)?;
    let mut vec = Vec::new();

    while let Some(v) = iter.next(env)? {
        vec.push(T::from_jni(env, v)?);
    }

    Ok(vec)
}

pub fn from_vec<T: Convert>(env: &mut JNIEnv, vec: Vec<T>) -> anyhow::Result<jobject> {
    let j_vec = env.new_object("java/util/ArrayList", "()V", &[])?;

    for v in vec {
        let value = v.into_jni(env)?;
        env.call_method(
            &j_vec,
            "add",
            "(Ljava/lang/Object;)Z",
            &[value.as_ref().into()],
        )?;
    }

    Ok(j_vec.into_raw())
}