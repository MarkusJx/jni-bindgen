# jni-bindgen

Automatically generate JNI bindings for Rust code.

## Usage

In order to use this crate, you need to add the following to your `Cargo.toml`:

```toml
[dependencies]
jni-bindgen = { git = "https://github.com/MarkusJx/jni-bindgen" }
```

If you want to generate Java bindings for your Rust code, you must set the
`JNI_BINDGEN_OUT_DIR` environment variable to the directory where the generated
code should be placed. Alternatively, you can use the cli tool to generate the
code.

### Export a struct to Java

```rust
use jni_bindgen::objects::traits::{FromJNI, ObjectFromJNI};
use jni_bindgen::jni;

struct MyClass {
    field: i32,
}

#[jni(package = "com.example")]
impl MyClass {
    #[jni(constructor)]
    fn ctor(value: i32) -> Self {
        Self { field: value }
    }
    
    #[jni]
    fn get_field(&self) -> i32 {
        self.field
    }
}
```

<details>
  <summary>Generated code</summary>

#### Rust code

```rust
#[no_mangle]
pub extern "system" fn Java_com_example_MyClass_00024MyClassNative_getField<'local>(
    mut env: jni::JNIEnv<'local>,
    object: jni::objects::JObject<'local>,
) -> jni::sys::jint {
    use jni_bindgen::objects::traits::IntoJNIResult;
    // Omitted: Retrieve this
    let res = this.get_field();
    res as jni::sys::jint
}

#[no_mangle]
pub extern "system" fn Java_com_example_MyClass_00024MyClassNative_ctor<'local>(
    mut env: jni::JNIEnv<'local>,
    class: jni::objects::JClass<'local>,
    j_arg_0: jni::sys::jint,
) -> jni::sys::jlong {
    use jni_bindgen::objects::traits::IntoJNIResult;
    let arg_0 = j_arg_0 as i32;
    let res = MyClass::ctor(arg_0);
    Box::into_raw(Box::new(res)) as jni::sys::jlong
}
```

#### Java code

```java
package com.example;

import com.github.markusjx.jnibindgen.NativeClass;
import com.github.markusjx.jnibindgen.NativeClassImpl;

public class MyClass implements NativeClassImpl<MyClass.MyClassNative> {
    private final MyClassNative inner;

    public MyClass(int value) {
        inner = new MyClassNative(value, this);
    }

    public int getField() {
        return inner.getField();
    }

    public static long getTypeHash() {
        return MyClassNative.getTypeHash();
    }

    @Override
    public MyClassNative getInner() {
        return inner;
    }

    public static class MyClassNative extends NativeClass {
        private MyClassNative(int value, Object referent) {
            super(ctor(value), referent);
        }

        private native int getField();

        private static native void drop(long self);

        private static native long getTypeHash();

        private static native long ctor(int value);

        @Override
        protected void destruct() {
            drop(this.ptr);
        }
    }
}
```

</details>

### Import an interface from Java

```rust
use jni_bindgen::objects::traits::{FromJNI, ObjectFromJNI};
use jni_bindgen::jni;
use jni::JNIEnv;

#[jni(package = "com.example")]
trait MyInterface {
    fn do_something(&self, env: &mut JNIEnv, value: i32) -> jni_bindgen::Result<i32>;
}

struct MyStruct;

#[jni(package = "com.example")]
impl MyStruct {
    #[jni]
    fn use_do_something<'a>(
        env: &mut JNIEnv,
        my_interface: Box<dyn MyInterface + 'a>,
        value: i32,
    ) -> jni_bindgen::Result<i32> {
        my_interface.do_something(env, value)
    }
}
```
