use crate::codegen::code::{load_library, outer_class, DESTRUCT};
use crate::codegen::java_method::JavaMethod;
use crate::codegen::traits::{AsDeclaration, FromDeclaration};
use crate::util::attrs::BindgenAttrs;
use crate::util::quotes;
use crate::util::traits::JniMethod;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::{ImplItem, ItemImpl};

pub struct JavaClass {
    pub name: String,
    pub methods: Vec<JavaMethod>,
    pub constructors: Vec<JavaMethod>,
    pub namespace: String,
    attrs: BindgenAttrs,
    _decl: ItemImpl,
}

impl JavaClass {
    pub fn from_declaration(decl: &ItemImpl, args: &BindgenAttrs) -> syn::Result<Self> {
        let name = decl.self_ty.clone().into_token_stream().to_string();
        let methods = decl
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(m) => {
                    if m.is_constructor() || !m.has_jni() {
                        None
                    } else {
                        Some(JavaMethod::from_declaration(m))
                    }
                }
                _ => None,
            })
            .collect::<syn::Result<Vec<_>>>()?;

        let constructors = decl
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(m) => {
                    if m.has_jni() && m.is_constructor() {
                        Some(JavaMethod::from_declaration(m))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect::<syn::Result<Vec<_>>>()?;

        Ok(Self {
            name,
            methods,
            constructors,
            attrs: args.clone(),
            namespace: args.get_namespace()?,
            _decl: decl.clone(),
        })
    }

    pub fn as_jni_methods(&self, args: &BindgenAttrs) -> syn::Result<TokenStream> {
        let namespace = args.get_namespace()?.replace('.', "_");
        let base_name = quotes::base_name(&namespace, &self.name);
        let methods: TokenStream = self
            .methods
            .iter()
            .map(|m| {
                m.as_jni_method(base_name.clone(), &self.name)
                    .map(|m| m.to_string())
            })
            .collect::<syn::Result<Vec<_>>>()?
            .join("\n")
            .parse()?;

        let constructors: TokenStream = self
            .constructors
            .iter()
            .map(|m| {
                m.as_jni_method(base_name.clone(), &self.name)
                    .map(|m| m.to_string())
            })
            .collect::<syn::Result<Vec<_>>>()?
            .join("\n")
            .parse()?;

        let drop = quotes::drop_struct(base_name.parse()?, self.name.clone());

        Ok(quote!(
            #methods
            #constructors
            #drop
        ))
    }
}

impl AsDeclaration for JavaClass {
    fn as_declaration(&self, _: bool) -> String {
        let mut methods_copy = self.methods.clone();
        methods_copy.push(JavaMethod::drop_method());
        methods_copy.append(&mut self.constructors.clone());

        let init_lib = if let Some(init) = self.attrs.load_lib() {
            load_library(&init)
        } else {
            String::new()
        };

        let throws =
            self.methods.iter().any(|m| m.throws()) || self.constructors.iter().any(|m| m.throws());

        let inner = format!(
            "public static class {}Native extends NativeClass {{\n{init_lib}\n{}\n\n{}\n\n{DESTRUCT}}}",
            self.name,
            methods_copy
                .iter()
                .map(|m| m.as_declaration(&self.name, false))
                .collect::<Vec<_>>()
                .join("\n\t"),
            self.constructors
                .iter()
                .map(|m| m.as_constructor(&format!("{}Native", self.name), true))
                .collect::<Vec<_>>()
                .join("\n\t"),
        );

        outer_class(
            &self.namespace,
            &self.name,
            self.methods
                .iter()
                .map(|m| m.as_declaration(&self.name, true))
                .collect::<Vec<_>>()
                .join("\n"),
            self.constructors
                .iter()
                .map(|m| m.as_constructor(&self.name, false))
                .collect::<Vec<_>>()
                .join("\n"),
            inner,
            throws,
        )
    }
}
