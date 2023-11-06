# jni-bindgen

Automatically generate JNI bindings for Rust code.

## Usage

### Export a struct to Java

```rust
use jni_bindgen::jni;

struct MyClass {
    field: i32,
}

#[jni(package = "com.example")]
impl MyClass {
    #[jni(constructor)]
    fn ctor(value: i32) {
        Self { field: value }
    }
    
    #[jni]
    fn get_field(&self) -> i32 {
        self.field
    }
}
```

### Import an interface from Java

```rust
use jni_bindgen::jni;
use jni::JNIEnv;

#[jni(package = "com.example")]
trait MyInterface {
    fn do_something(&self, env: &mut JNIEnv, value: i32) -> jni_bindgen::Result<i32>;
}

struct MyStruct;

#[jni(package = "com.example")]
impl MyStruct {
    fn use_do_something<'a>(
        &self,
        env: &mut JNIEnv,
        my_interface: Box<dyn MyInterface + 'a>,
        value: i32,
    ) -> jni_bindgen::Result<i32> {
        my_interface.do_something(env, value)
    }
}
```
