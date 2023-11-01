use crate::conversion::option_convert::*;
use crate::objects::traits::{FromJNI, IntoJNI};
use jni::objects::{JObject, JString};
use jni::sys::jobject;
use jni::JNIEnv;
use std::collections::HashMap;
use std::hash::Hash;

macro_rules! impl_convert {
    ($ty: ty, $from: ident, $to: ident) => {
        impl FromJNI<'_> for $ty {
            fn from_jni(env: &mut JNIEnv, obj: JObject) -> crate::Result<Self> {
                $from(env, obj)?.ok_or(crate::error!("The value is null"))
            }
        }

        impl IntoJNI for $ty {
            fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> crate::Result<JObject<'a>> {
                Ok(unsafe { JObject::from_raw($to(env, Some(self))?) })
            }
        }

        impl FromJNI<'_> for Option<$ty> {
            fn from_jni(env: &mut JNIEnv, obj: JObject) -> crate::Result<Self> {
                $from(env, obj)
            }
        }

        impl IntoJNI for Option<$ty> {
            fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> crate::Result<JObject<'a>> {
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

impl<'local> FromJNI<'local> for JObject<'local> {
    fn from_jni(_: &mut JNIEnv<'local>, obj: JObject) -> crate::Result<Self> {
        Ok(unsafe { JObject::from_raw(*obj) })
    }
}

impl IntoJNI for JObject<'_> {
    fn into_jni<'a>(self, _: &mut JNIEnv<'a>) -> crate::Result<JObject<'a>> {
        Ok(unsafe { JObject::from_raw(*self) })
    }
}

impl<'local> FromJNI<'local> for JString<'local> {
    fn from_jni(_: &mut JNIEnv<'local>, obj: JObject) -> crate::Result<Self> {
        Ok(unsafe { JString::from(JObject::from_raw(*obj)) })
    }
}

impl IntoJNI for JString<'_> {
    fn into_jni<'a>(self, _: &mut JNIEnv<'a>) -> crate::Result<JObject<'a>> {
        Ok(unsafe { JObject::from_raw(self.as_raw()) })
    }
}

impl<'local, K: FromJNI<'local> + Eq + Hash, V: FromJNI<'local>> FromJNI<'local> for HashMap<K, V> {
    fn from_jni(env: &mut JNIEnv<'local>, obj: JObject) -> crate::Result<Self> {
        into_hashmap(env, obj)
    }
}

impl<K: IntoJNI + Eq + Hash, V: IntoJNI> IntoJNI for HashMap<K, V> {
    fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> crate::Result<JObject<'a>> {
        Ok(unsafe { JObject::from_raw(from_hashmap(env, self)?) })
    }
}

impl<'local, K: FromJNI<'local> + Eq + Hash, V: FromJNI<'local>> FromJNI<'local>
    for Option<HashMap<K, V>>
{
    fn from_jni(env: &mut JNIEnv<'local>, obj: JObject) -> crate::Result<Self> {
        match obj.is_null() {
            true => Ok(None),
            false => Ok(Some(into_hashmap(env, obj)?)),
        }
    }
}

impl<K: IntoJNI + Eq + Hash, V: IntoJNI> IntoJNI for Option<HashMap<K, V>> {
    fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> crate::Result<JObject<'a>> {
        match self {
            Some(map) => Ok(unsafe { JObject::from_raw(from_hashmap(env, map)?) }),
            None => Ok(JObject::null()),
        }
    }
}

impl<'local, T: FromJNI<'local>> FromJNI<'local> for Vec<T> {
    fn from_jni(env: &mut JNIEnv<'local>, obj: JObject) -> crate::Result<Self> {
        into_vec(env, obj)
    }
}

impl<T: IntoJNI> IntoJNI for Vec<T> {
    fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> crate::Result<JObject<'a>> {
        Ok(unsafe { JObject::from_raw(from_vec(env, self)?) })
    }
}

impl<'local, T: FromJNI<'local>> FromJNI<'local> for Option<Vec<T>> {
    fn from_jni(env: &mut JNIEnv<'local>, obj: JObject) -> crate::Result<Self> {
        match obj.is_null() {
            true => Ok(None),
            false => Ok(Some(into_vec(env, obj)?)),
        }
    }
}

impl<T: IntoJNI> IntoJNI for Option<Vec<T>> {
    fn into_jni<'a>(self, env: &mut JNIEnv<'a>) -> crate::Result<JObject<'a>> {
        match self {
            Some(vec) => Ok(unsafe { JObject::from_raw(from_vec(env, vec)?) }),
            None => Ok(JObject::null()),
        }
    }
}

pub fn into_hashmap<'local, K: FromJNI<'local> + Eq + Hash, V: FromJNI<'local>>(
    env: &mut JNIEnv<'local>,
    obj: JObject,
) -> crate::Result<HashMap<K, V>> {
    let j_map = env.get_map(&obj)?;
    let mut iter = j_map.iter(env)?;
    let mut map = HashMap::new();

    while let Some((k, v)) = iter.next(env)? {
        map.insert(K::from_jni(env, k)?, V::from_jni(env, v)?);
    }

    Ok(map)
}

pub fn from_hashmap<K: IntoJNI + Eq + Hash, V: IntoJNI>(
    env: &mut JNIEnv,
    map: HashMap<K, V>,
) -> crate::Result<jobject> {
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

pub fn into_vec<'local, T: FromJNI<'local>>(
    env: &mut JNIEnv<'local>,
    obj: JObject,
) -> crate::Result<Vec<T>> {
    let j_vec = env.get_list(&obj)?;
    let mut iter = j_vec.iter(env)?;
    let mut vec = Vec::new();

    while let Some(v) = iter.next(env)? {
        vec.push(T::from_jni(env, v)?);
    }

    Ok(vec)
}

pub fn from_vec<T: IntoJNI>(env: &mut JNIEnv, vec: Vec<T>) -> crate::Result<jobject> {
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
