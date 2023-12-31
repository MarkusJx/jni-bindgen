use crate::codegen::code::interface;
use crate::codegen::java_method::JavaMethod;
use crate::codegen::traits::FromDeclaration;
use crate::util::attrs::BindgenAttrs;
use crate::util::traits::GetComment;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{ItemTrait, TraitItem};

pub struct JavaInterface {
    pub name: String,
    pub methods: Vec<JavaMethod>,
    pub namespace: String,
    //_attrs: BindgenAttrs,
    decl: ItemTrait,
}

impl JavaInterface {
    pub fn from_declaration(decl: &ItemTrait, args: &BindgenAttrs) -> syn::Result<Self> {
        let name = decl.ident.clone().into_token_stream().to_string();
        let methods = decl
            .items
            .iter()
            .map(|item| match item {
                TraitItem::Fn(func) => JavaMethod::from_declaration(func),
                _ => Err(syn::Error::new(item.span(), "Only functions are supported")),
            })
            .collect::<syn::Result<Vec<_>>>()?;

        Ok(Self {
            name,
            methods,
            //_attrs: args.clone(),
            namespace: args.get_namespace()?,
            decl: decl.clone(),
        })
    }

    pub fn as_jni_methods(&self) -> syn::Result<TokenStream> {
        let struct_name: TokenStream = format!("{}Impl", self.name).parse()?;
        let trait_name: TokenStream = self.name.parse()?;

        let methods = self
            .methods
            .iter()
            .map(|m| m.as_trait_method())
            .collect::<syn::Result<Vec<_>>>()?;

        Ok(quote! {
            pub struct #struct_name<'local> {
                pub obj: jni::objects::JObject<'local>,
            }

            #[automatically_derived]
            impl #trait_name for #struct_name<'_> {
                #(#methods)*
            }

            #[automatically_derived]
            impl<'local> FromJNI<'local> for Box<dyn #trait_name + 'local> {
                fn from_jni(
                    env: &mut jni::JNIEnv<'local>,
                    obj: jni::objects::JObject<'local>,
                ) -> jni_bindgen::Result<Self> {
                    Ok(Box::new(#struct_name { obj }))
                }
            }
        })
    }

    pub fn as_java_declaration(&self) -> String {
        let methods = self
            .methods
            .iter()
            .map(|m| m.as_interface_declaration())
            .collect::<Vec<_>>()
            .join("\n");

        let imports = self
            .methods
            .iter()
            .flat_map(|m| m.get_imports())
            .filter(|i| !i.contains("NativeExecutionException"))
            .collect::<HashSet<_>>();

        interface(
            &self.namespace,
            &self.name,
            self.decl.attrs.get_comment().unwrap_or_default(),
            methods,
            imports,
        )
    }
}
