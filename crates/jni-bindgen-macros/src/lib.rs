extern crate proc_macro;

use crate::util::expand;
use proc_macro::TokenStream;

mod codegen;
mod util;

/// This macro will generate the JNI bindings for the given class.
/// It can only be used on `impl` blocks and its methods.
///
/// If an `impl` block has this macro, jni methods for all methods
/// inside that block also annotated with `#[jni]` will be generated.
/// The associated `struct` must not be annotated with `#[jni]`.
///
/// # Arguments
/// * `namespace` - *Required* The namespace of the class.
/// * `load_lib` - The name of the library to load. If this is specified,
///               the library will be loaded using `System.loadLibrary`.
///              If this is not specified, the library will not be loaded automatically.
/// * `rename` - The name of the class. If this is specified, the (java) class will be renamed.
/// * `constructor` - If this is specified, the constructor will be renamed.
///
/// # Supported types
/// | Rust type | Java type |
/// |-----------|-----------|
/// | [`String`] | `java.lang.String` |
/// | [`Vec`] | `java.util.List` |
/// | [`HashMap`] | `java.util.Map` |
/// | [`Wrapped`](jni_bindgen::objects::wrapped::Wrapped) | The wrapped type |
/// | [`JObject`](jni::objects::JObject) | `java.lang.Object` |
/// | [`&JNIEnv`](jni::JNIEnv) | N/A |
/// | [`i32`] | `int` |
/// | [`i64`] | `long` |
/// | [`f32`] | `float` |
/// | [`f64`] | `double` |
/// | [`bool`] | `boolean` |
/// | [`i16`] | `short` |
/// | [`i8`] | `byte` |
/// | [`u16`] | `char` |
/// | [`Option<i32>`] | `java.lang.Integer` |
/// | [`Option<i64>`] | `java.lang.Long` |
/// | [`Option<f32>`] | `java.lang.Float` |
/// | [`Option<f64>`] | `java.lang.Double` |
/// | [`Option<bool>`] | `java.lang.Boolean` |
/// | [`Option<i16>`] | `java.lang.Short` |
/// | [`Option<i8>`] | `java.lang.Byte` |
/// | [`Option<u16>`] | `java.lang.Character` |
/// | [`Option<String>`] | `java.lang.String` |
/// | Any other [`Option`] | The wrapped type |
///
/// # Returning errors
/// If a method returns a [`Result`], the error will be converted into a JNI error.
/// You can return any error which can be converted into a [`String`]. In this case,
/// the error thrown will be of type `com.github.markusjx.jnibindgen.NativeExecutionException`.
/// The error will be added to the generated java method signature.
///
/// If you want to throw a custom exception, you can use the [`bail_class!`](jni_bindgen::bail_class)
/// or [`error_class!`](jni_bindgen::error_class) macros while returning a [`jni_bindgen::Result<T>`].
///
/// # Example
/// ```
/// use jni_bindgen_macros::jni;
///
/// struct MyClass {
///     value: String,
/// }
///
/// #[jni(namespace = "com.github.markusjx.generated")]
/// impl RustStruct {
///     #[jni(constructor, rename = "newInstance")]
///     fn new(value: String) -> Self {
///         Self { value }
///     }
///
///     #[jni]
///     fn get_value(&self) -> String {
///         self.value.clone()
///     }
///
///     #[jni]
///     fn throws_error() -> anyhow::Result<()> {
///         Err(anyhow::anyhow!("Error"))
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn jni(args: TokenStream, input: TokenStream) -> TokenStream {
    expand::expand(args, input)
        .map_err(|e| e.to_compile_error())
        .unwrap_or_else(|e| e.into())
}
