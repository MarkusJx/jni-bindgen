use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn this(struct_name: &TokenStream, ret_val: &TokenStream) -> TokenStream {
    quote!(
        let this = match env.get_field(object, "ptr", "J")
            .and_then(|e| e.j())
            .map_err(|e| e.to_string())
            .and_then(|ptr| {
                if ptr == 0 {
                    Err("The pointer is null".to_string())
                } else {
                    Ok(unsafe { &*(ptr as *const #struct_name) })
                }
            })
            .into_jni_result() {
                Ok(this) => this,
                Err(e) => {
                    e.throw(&mut env);
                    return #ret_val;
                }
            };
    )
}

pub fn base_name(namespace: &str, struct_name: &str) -> String {
    [
        "Java".to_string(),
        namespace.to_string(),
        struct_name.to_string(),
        format!("00024{}Native", struct_name),
    ]
    .join("_")
}

pub fn call(
    has_self: bool,
    has_ret: bool,
    method_name: &Ident,
    struct_name: &TokenStream,
    call_args: TokenStream,
) -> TokenStream {
    if has_self && has_ret {
        quote!(let res = this.#method_name(#call_args))
    } else if has_self {
        quote!(this.#method_name(#call_args))
    } else if has_ret {
        quote!(let res = #struct_name::#method_name(#call_args))
    } else {
        quote!(#struct_name::#method_name(#call_args))
    }
}

pub fn drop_struct(base_name: TokenStream, struct_name: String) -> TokenStream {
    let drop_name: TokenStream = format!("{base_name}_drop").parse().unwrap();
    let struct_name: TokenStream = struct_name.parse().unwrap();

    quote!(
        #[no_mangle]
        pub extern "system" fn #drop_name<'local>(
            _env: jni::JNIEnv<'local>,
            _class: jni::objects::JClass<'local>,
            ptr: jni::sys::jlong
        ) {
            unsafe {
                if ptr != 0 {
                    drop(Box::from_raw(ptr as *mut #struct_name));
                }
            }
        }
    )
}

pub fn get_type_hash(base_name: TokenStream, struct_name: String) -> TokenStream {
    let get_type_hash: TokenStream = format!("{base_name}_getTypeHash").parse().unwrap();
    let struct_name: TokenStream = struct_name.parse().unwrap();

    quote!(
        #[no_mangle]
        pub extern "system" fn #get_type_hash<'local>(
            mut env: jni::JNIEnv<'local>,
            _class: jni::objects::JClass<'local>,
        ) -> jni::sys::jlong {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            TypeId::of::<#struct_name>().hash(&mut hasher);

            hasher.finish() as jni::sys::jlong
        }
    )
}
