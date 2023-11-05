use jni::JNIEnv;
use jni_bindgen::objects::traits::{FromJNI, ObjectFromJNI};
use std::collections::HashMap;

#[jni(package = "com.github.markusjx.generated")]
/// Trait used for testing
pub trait ApplyString {
    /// Return the input string
    ///
    /// @param val The input string
    /// @return The input string
    fn apply(&self, env: &mut JNIEnv, val: String) -> jni_bindgen::Result<String>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyStringOpt {
    fn apply(&self, env: &mut JNIEnv, val: Option<String>) -> jni_bindgen::Result<Option<String>>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyStringVec {
    fn apply(&self, env: &mut JNIEnv, val: Vec<String>) -> jni_bindgen::Result<Vec<String>>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyStringVecOpt {
    fn apply(
        &self,
        env: &mut JNIEnv,
        val: Option<Vec<String>>,
    ) -> jni_bindgen::Result<Option<Vec<String>>>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyStringHashMap {
    fn apply(
        &self,
        env: &mut JNIEnv,
        val: HashMap<String, String>,
    ) -> jni_bindgen::Result<HashMap<String, String>>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyStringHashMapOpt {
    fn apply(
        &self,
        env: &mut JNIEnv,
        val: Option<HashMap<String, String>>,
    ) -> jni_bindgen::Result<Option<HashMap<String, String>>>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyInt {
    fn apply(&self, env: &mut JNIEnv, val: i32) -> jni_bindgen::Result<i32>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyIntOpt {
    fn apply(&self, env: &mut JNIEnv, val: Option<i32>) -> jni_bindgen::Result<Option<i32>>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyLong {
    fn apply(&self, env: &mut JNIEnv, val: i64) -> jni_bindgen::Result<i64>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyLongOpt {
    fn apply(&self, env: &mut JNIEnv, val: Option<i64>) -> jni_bindgen::Result<Option<i64>>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyFloat {
    fn apply(&self, env: &mut JNIEnv, val: f32) -> jni_bindgen::Result<f32>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyFloatOpt {
    fn apply(&self, env: &mut JNIEnv, val: Option<f32>) -> jni_bindgen::Result<Option<f32>>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyDouble {
    fn apply(&self, env: &mut JNIEnv, val: f64) -> jni_bindgen::Result<f64>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyDoubleOpt {
    fn apply(&self, env: &mut JNIEnv, val: Option<f64>) -> jni_bindgen::Result<Option<f64>>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyBool {
    fn apply(&self, env: &mut JNIEnv, val: bool) -> jni_bindgen::Result<bool>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyBoolOpt {
    fn apply(&self, env: &mut JNIEnv, val: Option<bool>) -> jni_bindgen::Result<Option<bool>>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyByte {
    fn apply(&self, env: &mut JNIEnv, val: i8) -> jni_bindgen::Result<i8>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyByteOpt {
    fn apply(&self, env: &mut JNIEnv, val: Option<i8>) -> jni_bindgen::Result<Option<i8>>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyChar {
    fn apply(&self, env: &mut JNIEnv, val: u16) -> jni_bindgen::Result<u16>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyCharOpt {
    fn apply(&self, env: &mut JNIEnv, val: Option<u16>) -> jni_bindgen::Result<Option<u16>>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyShort {
    fn apply(&self, env: &mut JNIEnv, val: i16) -> jni_bindgen::Result<i16>;
}

#[jni(package = "com.github.markusjx.generated")]
pub trait ApplyShortOpt {
    fn apply(&self, env: &mut JNIEnv, val: Option<i16>) -> jni_bindgen::Result<Option<i16>>;
}

struct StructUsingTrait;

#[jni(package = "com.github.markusjx.generated", load_lib = "example_lib")]
/// Struct using the trait
impl StructUsingTrait {
    #[jni]
    /// Use the trait
    ///
    /// @param traitObj The trait object
    /// @return The input string
    fn use_apply_string<'a>(
        trait_obj: Box<dyn ApplyString + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<String> {
        trait_obj.apply(env, "test".to_string())
    }

    #[jni]
    fn use_apply_string_opt<'a>(
        trait_obj: Box<dyn ApplyStringOpt + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<Option<String>> {
        trait_obj.apply(env, Some("test".to_string()))
    }

    #[jni]
    fn use_apply_string_vec<'a>(
        trait_obj: Box<dyn ApplyStringVec + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<Vec<String>> {
        trait_obj.apply(env, vec!["test".to_string()])
    }

    #[jni]
    fn use_apply_string_vec_opt<'a>(
        trait_obj: Box<dyn ApplyStringVecOpt + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<Option<Vec<String>>> {
        trait_obj.apply(env, Some(vec!["test".to_string()]))
    }

    #[jni]
    fn use_apply_string_hashmap<'a>(
        trait_obj: Box<dyn ApplyStringHashMap + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<HashMap<String, String>> {
        let mut map = HashMap::new();
        map.insert("test".to_string(), "test".to_string());
        trait_obj.apply(env, map)
    }

    #[jni]
    fn use_apply_string_hashmap_opt<'a>(
        trait_obj: Box<dyn ApplyStringHashMapOpt + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<Option<HashMap<String, String>>> {
        let mut map = HashMap::new();
        map.insert("test".to_string(), "test".to_string());
        trait_obj.apply(env, Some(map))
    }

    #[jni]
    fn use_apply_int<'a>(
        trait_obj: Box<dyn ApplyInt + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<i32> {
        trait_obj.apply(env, 1)
    }

    #[jni]
    fn use_apply_int_opt<'a>(
        trait_obj: Box<dyn ApplyIntOpt + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<Option<i32>> {
        trait_obj.apply(env, Some(1))
    }

    #[jni]
    fn use_apply_long<'a>(
        trait_obj: Box<dyn ApplyLong + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<i64> {
        trait_obj.apply(env, 1)
    }

    #[jni]
    fn use_apply_long_opt<'a>(
        trait_obj: Box<dyn ApplyLongOpt + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<Option<i64>> {
        trait_obj.apply(env, Some(1))
    }

    #[jni]
    fn use_apply_float<'a>(
        trait_obj: Box<dyn ApplyFloat + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<f32> {
        trait_obj.apply(env, 1.0)
    }

    #[jni]
    fn use_apply_float_opt<'a>(
        trait_obj: Box<dyn ApplyFloatOpt + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<Option<f32>> {
        trait_obj.apply(env, Some(1.0))
    }

    #[jni]
    fn use_apply_double<'a>(
        trait_obj: Box<dyn ApplyDouble + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<f64> {
        trait_obj.apply(env, 1.0)
    }

    #[jni]
    fn use_apply_double_opt<'a>(
        trait_obj: Box<dyn ApplyDoubleOpt + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<Option<f64>> {
        trait_obj.apply(env, Some(1.0))
    }

    #[jni]
    fn use_apply_bool<'a>(
        trait_obj: Box<dyn ApplyBool + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<bool> {
        trait_obj.apply(env, true)
    }

    #[jni]
    fn use_apply_bool_opt<'a>(
        trait_obj: Box<dyn ApplyBoolOpt + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<Option<bool>> {
        trait_obj.apply(env, Some(true))
    }

    #[jni]
    fn use_apply_byte<'a>(
        trait_obj: Box<dyn ApplyByte + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<i8> {
        trait_obj.apply(env, 1)
    }

    #[jni]
    fn use_apply_byte_opt<'a>(
        trait_obj: Box<dyn ApplyByteOpt + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<Option<i8>> {
        trait_obj.apply(env, Some(1))
    }

    #[jni]
    fn use_apply_char<'a>(
        trait_obj: Box<dyn ApplyChar + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<u16> {
        trait_obj.apply(env, 'a' as u16)
    }

    #[jni]
    fn use_apply_char_opt<'a>(
        trait_obj: Box<dyn ApplyCharOpt + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<Option<u16>> {
        trait_obj.apply(env, Some('a' as u16))
    }

    #[jni]
    fn use_apply_short<'a>(
        trait_obj: Box<dyn ApplyShort + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<i16> {
        trait_obj.apply(env, 1)
    }

    #[jni]
    fn use_apply_short_opt<'a>(
        trait_obj: Box<dyn ApplyShortOpt + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<Option<i16>> {
        trait_obj.apply(env, Some(1))
    }
}
