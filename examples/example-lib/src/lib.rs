use anyhow::bail;
use jni::objects::{JObject, JValue, JValueGen, JValueOwned};
use jni::JNIEnv;
use jni_bindgen::errors::jni_error::ErrorClass;
use jni_bindgen::jni;
use jni_bindgen::objects::traits::{FromJNI, ObjectFromJNI};
use std::any::TypeId;
use std::collections::HashMap;

#[jni(namespace = "com.github.markusjx.generated")]
trait TestTrait {
    fn test(&self, env: &mut JNIEnv, val: String) -> jni_bindgen::Result<String>;

    fn other(&self, env: &mut JNIEnv) -> jni_bindgen::Result<String>;
}

struct RustStruct {
    value: String,
}

impl Drop for RustStruct {
    fn drop(&mut self) {
        println!("Dropping RustStruct with value {}", self.value);
    }
}

struct NativeStruct;

#[jni(namespace = "com.github.markusjx.generated", load_lib = "example_lib")]
impl NativeStruct {
    #[jni]
    fn get_rust_struct_value(rust_struct: &RustStruct) -> String {
        rust_struct.value.clone()
    }

    #[jni]
    fn get_rust_struct_value_opt(opt: Option<&RustStruct>) -> Option<String> {
        opt.map(|s| s.value.clone())
    }

    #[jni]
    fn get_obj(obj: JObject) -> JObject {
        obj
    }

    #[jni]
    fn get_vec(vec: Vec<String>) -> Vec<String> {
        vec
    }

    #[jni]
    fn get_vec_opt(opt: Option<Vec<String>>) -> Option<Vec<String>> {
        opt
    }

    #[jni]
    fn get_hashmap(map: HashMap<String, String>) -> HashMap<String, String> {
        map
    }

    #[jni]
    fn get_hashmap_opt(opt: Option<HashMap<String, String>>) -> Option<HashMap<String, String>> {
        opt
    }

    #[jni]
    fn get_vec_values(vec: Vec<&RustStruct>) -> Vec<String> {
        vec.into_iter().map(|s| s.get_value()).collect()
    }
}

#[jni(namespace = "com.github.markusjx.generated", load_lib = "example_lib")]
impl RustStruct {
    #[jni(constructor, rename = "initSingle")]
    /// Create a new RustStruct with the given value
    ///
    /// @param value The value to use
    fn new(value: String) -> Self {
        Self { value }
    }

    #[jni]
    fn get_value(&self) -> String {
        self.value.clone()
    }

    #[jni]
    fn set_value(&mut self, value: String) {
        self.value = value;
    }

    #[jni]
    fn get_string(opt: Option<String>) -> Option<String> {
        opt
    }

    #[jni]
    fn get_int(opt: Option<i32>) -> Option<i32> {
        opt
    }

    #[jni]
    fn get_long(opt: Option<i64>) -> Option<i64> {
        opt
    }

    #[jni]
    fn get_float(opt: Option<f32>) -> Option<f32> {
        opt
    }

    #[jni]
    fn get_double(opt: Option<f64>) -> Option<f64> {
        opt
    }

    #[jni]
    fn get_bool(opt: Option<bool>) -> Option<bool> {
        opt
    }

    #[jni]
    fn get_byte(opt: Option<i8>) -> Option<i8> {
        opt
    }

    #[jni]
    fn get_char(opt: Option<u16>) -> Option<u16> {
        opt
    }

    #[jni]
    fn get_short(opt: Option<i16>) -> Option<i16> {
        opt
    }

    #[jni]
    fn throw_error(msg: String) -> anyhow::Result<()> {
        bail!(msg)
    }

    #[jni]
    fn throw_other_error(err: String, msg: String) -> jni_bindgen::Result<()> {
        jni_bindgen::bail_class!(ErrorClass::Any(err), "{}", msg)
    }
}
