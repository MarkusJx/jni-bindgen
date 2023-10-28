use crate::codegen::java_class::JavaClass;
use crate::codegen::traits::AsDeclaration;
use crate::util::attrs::BindgenAttrs;
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::Item;

pub fn expand(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let item = syn::parse2::<Item>(input.into())?;
    let args: BindgenAttrs = syn::parse2(args.into())?;

    let code = match item.clone() {
        Item::Struct(str) => {
            return Err(syn::Error::new(str.span(), "structs are not supported"));
        }
        Item::Impl(impl_) => {
            let java_class = JavaClass::from_declaration(&impl_, &args)?;
            let java_class_decl = java_class.as_declaration(false);

            let res = java_class.as_jni_methods(&args)?;
            if let Ok(debug) = std::env::var("DEBUG_JNI_BINDGEN") {
                if debug == "true" {
                    println!("{java_class_decl}\n\n{res}");
                }
            }

            Some(res)
        }
        _ => None,
    };

    Ok(quote!(
        #item

        #code
    )
    .into())
}
