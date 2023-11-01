use jni::objects::JString;
macro_rules! impl_option_into_jni {
    ($t:ty, $to: ident, $from: ident, $cls: expr, $constructor: expr, $getter: expr, $get_fn: ident) => {
        pub fn $to(env: &mut jni::JNIEnv, val: Option<$t>) -> crate::Result<jni::sys::jobject> {
            Ok(match val {
                Some(val) => env
                    .call_static_method(
                        $cls,
                        "valueOf",
                        format!("({})L{};", $constructor, $cls),
                        &[val.into()],
                    )?
                    .l()?
                    .into_raw(),
                None => std::ptr::null_mut(),
            })
        }

        pub fn $from(
            env: &mut jni::JNIEnv,
            val: jni::objects::JObject,
        ) -> crate::Result<Option<$t>> {
            Ok(if val.is_null() {
                None
            } else {
                Some(
                    env.call_method(val, $getter, format!("(){}", $constructor), &[])?
                        .$get_fn()? as $t,
                )
            })
        }
    };
}

impl_option_into_jni!(
    i32,
    i32_into_jni,
    i32_from_jni,
    "java/lang/Integer",
    "I",
    "intValue",
    i
);
impl_option_into_jni!(
    i64,
    i64_into_jni,
    i64_from_jni,
    "java/lang/Long",
    "J",
    "longValue",
    j
);
impl_option_into_jni!(
    f32,
    f32_into_jni,
    f32_from_jni,
    "java/lang/Float",
    "F",
    "floatValue",
    f
);
impl_option_into_jni!(
    f64,
    f64_into_jni,
    f64_from_jni,
    "java/lang/Double",
    "D",
    "doubleValue",
    d
);
impl_option_into_jni!(
    bool,
    bool_into_jni,
    bool_from_jni,
    "java/lang/Boolean",
    "Z",
    "booleanValue",
    z
);
impl_option_into_jni!(
    i16,
    i16_into_jni,
    i16_from_jni,
    "java/lang/Short",
    "S",
    "shortValue",
    s
);
impl_option_into_jni!(
    u16,
    u16_into_jni,
    u16_from_jni,
    "java/lang/Character",
    "C",
    "charValue",
    c
);
impl_option_into_jni!(
    i8,
    i8_into_jni,
    i8_from_jni,
    "java/lang/Byte",
    "B",
    "byteValue",
    b
);

pub fn string_into_jni(
    env: &mut jni::JNIEnv,
    val: Option<String>,
) -> crate::Result<jni::sys::jobject> {
    Ok(match val {
        Some(val) => env.new_string(val)?.into_raw(),
        None => std::ptr::null_mut(),
    })
}

pub fn string_from_jni(
    env: &mut jni::JNIEnv,
    val: jni::objects::JObject,
) -> crate::Result<Option<String>> {
    Ok(if val.is_null() {
        None
    } else {
        Some(env.get_string(&JString::from(val))?.into())
    })
}
