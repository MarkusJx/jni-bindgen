use anyhow::bail;
use jni_bindgen::jni;

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
    #[jni(constructor)]
    fn create() -> Self {
        Self
    }

    #[jni]
    fn get_rust_struct_value(rust_struct: &RustStruct) -> String {
        rust_struct.value.clone()
    }

    #[jni]
    fn set_rust_struct_value(rust_struct: &mut RustStruct, value: String) {
        rust_struct.value = value;
    }

    #[jni]
    fn get_rust_struct_value_opt(opt: Option<&RustStruct>) -> Option<String> {
        opt.map(|s| s.value.clone())
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
}
