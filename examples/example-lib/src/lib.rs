use anyhow::bail;
use jni_bindgen_macros::jni;

struct RustStruct {
    value: String,
}

impl Drop for RustStruct {
    fn drop(&mut self) {
        println!("Dropping RustStruct with value {}", self.value);
    }
}

#[jni(namespace = "com.github.markusjx", load_lib = "java_native")]
impl RustStruct {
    #[jni(constructor, rename = "initSingle")]
    /// Create a new RustStruct with the given value
    ///
    /// @param value The value to use
    fn new(value: String) -> Self {
        Self { value }
    }

    #[jni(constructor)]
    fn new2() -> Self {
        Self {
            value: "Hello World".to_string(),
        }
    }

    #[jni(constructor)]
    fn new3(_v1: String, _v2: String) -> anyhow::Result<Self> {
        bail!("Error")
    }

    #[jni]
    fn print(&self) {
        println!("Value: {}", self.value);
    }

    #[jni]
    fn empty_func(_args_name: String) -> &'static str {
        println!("Empty");
        "args_name"
    }

    #[jni]
    fn with_result(&self) -> anyhow::Result<()> {
        bail!("Error")
    }

    #[jni]
    fn numeric(&self, val: i32, _env: &mut jni::JNIEnv, _val2: String) -> i32 {
        val + 1
    }
}
