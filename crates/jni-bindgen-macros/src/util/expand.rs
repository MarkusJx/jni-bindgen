use crate::codegen::java_class::JavaClass;
use crate::codegen::java_interface::JavaInterface;
use crate::util::attrs::BindgenAttrs;
use crate::util::traits::AnyAttribute;
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{FnArg, Item};

fn write_class(out_dir: &Option<String>, namespace: &str, class_name: &str, decl: &str) {
    if let Some(java_dir) = out_dir.as_ref() {
        let java_file = std::path::Path::new(&java_dir)
            .join(namespace.replace('.', "/"))
            .join(format!("{class_name}.java"));

        std::fs::create_dir_all(java_file.parent().unwrap()).unwrap();
        std::fs::write(java_file, decl).unwrap();
    }
}

pub fn expand(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let item = syn::parse2::<Item>(input.into())?;
    let args: BindgenAttrs = syn::parse2(args.into())?;

    let mut debug = false;
    if let Ok(env_debug) = std::env::var("DEBUG_JNI_BINDGEN") {
        if env_debug == "true" {
            debug = true;
        }
    }

    let out_dir = std::env::var("JNI_BINDGEN_OUT_DIR").ok();

    let code = match item.clone() {
        Item::Impl(impl_) => {
            let java_class = JavaClass::from_declaration(&impl_, &args)?;

            let res = java_class.as_jni_methods(&args)?;
            let java_class_decl = java_class.as_declaration();
            if debug {
                println!("{java_class_decl}\n\n{res}");
            }

            write_class(
                &out_dir,
                &java_class.namespace,
                &java_class.name,
                &java_class_decl,
            );
            Some(res)
        }
        Item::Trait(tr) => {
            let interface = JavaInterface::from_declaration(&tr, &args)?;
            let res = interface.as_jni_methods()?;
            let java_decl = interface.as_java_declaration();
            if debug {
                println!("{res}\n\n{java_decl}");
            }

            write_class(&out_dir, &interface.namespace, &interface.name, &java_decl);
            Some(res)
        }
        Item::Fn(mut func) => {
            func.sig.inputs = func
                .sig
                .inputs
                .into_iter()
                .map(|i| match i {
                    FnArg::Typed(mut typed) => {
                        typed.attrs.retain(|a| !a.is_jni());
                        FnArg::Typed(typed)
                    }
                    rest => rest,
                })
                .collect();

            return Ok(quote!(#func).into());
        }
        Item::Verbatim(_) => None,
        _ => {
            return Err(syn::Error::new(
                item.span(),
                "Only impl blocks and functions are supported",
            ))
        }
    };

    Ok(quote!(
        #item

        #code
    )
    .into())
}
