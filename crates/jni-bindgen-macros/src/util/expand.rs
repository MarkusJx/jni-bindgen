use crate::codegen::java_class::JavaClass;
use crate::util::attrs::BindgenAttrs;
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::Item;

pub fn expand(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let item = syn::parse2::<Item>(input.into())?;
    let args: BindgenAttrs = syn::parse2(args.into())?;

    let code = match item.clone() {
        Item::Impl(impl_) => {
            let java_class = JavaClass::from_declaration(&impl_, &args)?;

            let res = java_class.as_jni_methods(&args)?;
            let java_class_decl = java_class.as_declaration();
            if let Ok(debug) = std::env::var("DEBUG_JNI_BINDGEN") {
                if debug == "true" {
                    println!("{java_class_decl}\n\n{res}");
                }
            }

            if let Ok(java_dir) = std::env::var("JNI_BINDGEN_OUT_DIR") {
                let java_file = std::path::Path::new(&java_dir)
                    .join(java_class.namespace.replace('.', "/"))
                    .join(format!("{}.java", java_class.name));

                std::fs::create_dir_all(java_file.parent().unwrap()).unwrap();
                std::fs::write(java_file, java_class_decl).unwrap();
            }

            Some(res)
        }
        Item::Fn(_) => None,
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
