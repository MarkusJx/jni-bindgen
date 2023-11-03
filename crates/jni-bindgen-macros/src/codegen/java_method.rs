use crate::codegen::java_type::{JNIArgGetter, JavaArg, JavaType};
use crate::codegen::traits::FromDeclaration;
use crate::util::quotes;
use crate::util::traits::JniMethod;
use convert_case::{Case, Casing};
use indexmap::IndexMap;
use proc_macro2::Ident;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use quote::ToTokens;
use std::collections::HashSet;
use std::ptr::addr_of;
use syn::spanned::Spanned;
use syn::{Attribute, Expr, FnArg, Lit, PatType, Signature, TraitItemFn};
use syn::{ImplItemFn, Meta};

#[derive(Clone)]
pub struct JavaMethod {
    pub name: String,
    pub original_name: String,
    pub args: IndexMap<String, JavaArg>,
    pub return_type: Option<JavaType>,
    pub static_method: bool,
    pub mut_self: bool,
    _decl: Option<ImplOrTraitFn>,
}

#[derive(Clone)]
enum ImplOrTraitFn {
    Impl(ImplItemFn),
    Trait(TraitItemFn),
}

impl ImplOrTraitFn {
    fn attrs(&self) -> &Vec<Attribute> {
        match self {
            Self::Impl(i) => &i.attrs,
            Self::Trait(t) => &t.attrs,
        }
    }
}

impl JavaMethod {
    pub fn drop_method() -> Self {
        Self {
            name: "drop".to_string(),
            original_name: "drop".to_string(),
            args: vec![("self".to_string(), JavaArg::this())]
                .into_iter()
                .collect(),
            return_type: None,
            static_method: true,
            mut_self: false,
            _decl: None,
        }
    }

    pub fn get_type_hash() -> Self {
        Self {
            name: "getTypeHash".to_string(),
            original_name: "getTypeHash".to_string(),
            args: Default::default(),
            return_type: Some(JavaType::Long),
            static_method: true,
            mut_self: false,
            _decl: None,
        }
    }

    pub fn get_imports(&self) -> HashSet<String> {
        let mut res = self
            .args
            .values()
            .flat_map(|a| a.java_type.get_imports())
            .collect::<HashSet<String>>();

        if let Some(ret) = self.return_type.as_ref() {
            res.extend(ret.get_imports());
        }

        res
    }

    fn get_name(arg: &FnArg) -> String {
        match arg {
            FnArg::Receiver(_) => "self".to_string(),
            FnArg::Typed(PatType { pat, .. }) => pat.into_token_stream().to_string(),
        }
    }

    fn get_args(&self) -> String {
        self.args
            .iter()
            .filter_map(|(name, arg)| {
                Some(format!(
                    "{} {}",
                    arg.as_declaration()?,
                    name.to_case(Case::Camel)
                ))
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn get_arg_names(&self) -> String {
        self.args
            .iter()
            .filter_map(|(n, a)| {
                if a.as_declaration().is_some() {
                    Some(n.to_case(Case::Camel))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn as_constructor(&self, class_name: &str, inner: bool) -> String {
        let code = if inner {
            format!(
                "super({}({}), referent);",
                self.name.to_case(Case::Camel),
                self.get_arg_names()
            )
        } else {
            let mut args = self.get_arg_names();
            if !args.is_empty() {
                args += ", ";
            }

            format!("inner = new {class_name}Native({args}this);")
        };

        let mut args = self.get_args();
        if inner {
            if !args.is_empty() {
                args += ", ";
            }

            args += "Object referent";
        }

        let public = if inner { "private" } else { "public" };
        let throws = self
            .return_type
            .as_ref()
            .and_then(|r| r.throws())
            .unwrap_or_default();
        let comment = self.get_comment().unwrap_or_default();

        format!(
            r#"{comment}{public} {class_name}({args}){throws} {{
        {code}
        }}
        "#
        )
    }

    pub fn as_declaration(&self, struct_name: &str, definition: bool) -> String {
        let static_method = if self.static_method { "static " } else { "" };
        let return_type = match &self.return_type {
            Some(t) => t.as_declaration().unwrap(),
            None => "void".to_string(),
        };

        let native = if definition { "" } else { "native " };
        let def = if definition {
            let ret = if self
                .return_type
                .as_ref()
                .map(|r| !r.is_void(true))
                .unwrap_or_default()
            {
                "return "
            } else {
                ""
            };

            let inner = if self.static_method {
                format!("{struct_name}Native")
            } else {
                "inner".to_string()
            };

            format!(
                " {{\n\t\t{ret}{inner}.{}({});\n\t}}",
                self.name.to_case(Case::Camel),
                self.get_arg_names()
            )
        } else {
            ";".to_string()
        };

        let public = if definition { "public" } else { "private" };
        let throws = self
            .return_type
            .as_ref()
            .and_then(|r| r.throws())
            .unwrap_or_default();
        let comment = self.get_comment().unwrap_or_default();

        format!(
            "\t{comment}{public} {static_method}{native}{return_type} {}({}){throws}{def}",
            self.name.to_case(Case::Camel),
            self.get_args()
        )
    }

    pub fn as_jni_method(&self, base_name: String, struct_name: &str) -> syn::Result<TokenStream> {
        let name: TokenStream = [base_name.to_string(), self.name.to_case(Case::Camel)]
            .join("_")
            .parse()?;

        let ret = self
            .return_type
            .as_ref()
            .map(|r| -> syn::Result<TokenStream> {
                let ret = r.as_jni_return_type()?;
                Ok(quote!(-> #ret))
            })
            .map_or(Ok(None), |v| v.map(Some))?;

        let parsed_struct_name = struct_name.parse()?;
        let this = if self.static_method {
            None
        } else {
            let ret_val = self
                .return_type
                .as_ref()
                .map_or(Ok(quote!(())), |r| r.error_return_val())?;
            Some(quotes::this(&parsed_struct_name, &ret_val, self.mut_self))
        };

        let j_args: TokenStream = self
            .args
            .values()
            .filter(|a| !a.is_self())
            .enumerate()
            .filter_map(|(i, a)| {
                a.as_jni_fn_arg()
                    .map_or_else(|e| Some(Err(e)), |a| a.map(|o| Ok((i, o))))
            })
            .collect::<syn::Result<Vec<_>>>()?
            .into_iter()
            .map(|(i, a)| {
                let name: TokenStream = format!("j_arg_{i}").parse()?;
                Ok(quote!(#name: #a).to_string())
            })
            .collect::<syn::Result<Vec<_>>>()?
            .join(", ")
            .parse()?;

        let arg_converters: TokenStream = self
            .args
            .values()
            .filter(|a| !a.is_self())
            .enumerate()
            .map(|(i, a)| {
                let name = format!("j_arg_{i}");
                let arg_name: TokenStream = format!("arg_{i}").parse()?;
                let getter = a.as_jni_arg_getter(&name, self.return_type.as_ref())?;

                if let JNIArgGetter::Getter(getter) = getter {
                    Ok(Some(
                        quote!(
                            let #arg_name = #getter;
                        )
                        .to_string(),
                    ))
                } else {
                    Ok(None)
                }
            })
            .filter_map(|a| a.map_or_else(|e| Some(Err(e)), |a| a.map(Ok)))
            .collect::<syn::Result<Vec<_>>>()?
            .join("\n")
            .parse()?;

        let args: TokenStream = self
            .args
            .values()
            .filter(|a| !a.is_self())
            .enumerate()
            .map(|(i, a)| {
                Ok(
                    match a.as_jni_arg_getter("arg", self.return_type.as_ref())? {
                        JNIArgGetter::Getter(_) => format!("arg_{i}"),
                        JNIArgGetter::ArgName(name) => name,
                    },
                )
            })
            .collect::<syn::Result<Vec<_>>>()?
            .join(", ")
            .parse()?;

        let class_or_this = if this.is_some() {
            quote!(object: jni::objects::JObject<'local>)
        } else {
            quote!(class: jni::objects::JClass<'local>)
        };

        let method_name = Ident::new(&self.original_name, Span::call_site());
        let call = quotes::call(
            this.is_some(),
            ret.is_some() && !self.return_type.as_ref().unwrap().is_void(false),
            &method_name,
            &parsed_struct_name,
            args,
        );

        let return_res = self
            .return_type
            .as_ref()
            .map_or(Ok(quote!()), |r| r.as_jni_return_val())?;

        Ok(quote!(
            #[no_mangle]
            pub extern "system" fn #name<'local>(
                mut env: jni::JNIEnv<'local>,
                #class_or_this,
                #j_args
            ) #ret {
                use jni_bindgen::objects::traits::IntoJNIResult;
                #this
                #arg_converters

                #call;
                #return_res
            }
        ))
    }

    pub fn as_trait_method(&self) -> syn::Result<TokenStream> {
        let name: TokenStream = self.name.parse()?;
        let ImplOrTraitFn::Trait(decl) = self._decl.as_ref().unwrap() else {
            panic!("Expected trait method")
        };

        if self.static_method {
            return Err(syn::Error::new(
                decl.span(),
                "Trait methods cannot be static",
            ));
        }

        if self.return_type.is_none()
            || !matches!(self.return_type.as_ref().unwrap(), JavaType::Result { .. })
        {
            return Err(syn::Error::new(
                decl.span(),
                "Trait methods must return a Result",
            ));
        }

        match self.args.values().find_map(|a| match &a.java_type {
            JavaType::Env { mutable, inner } => Some((mutable, inner)),
            _ => None,
        }) {
            Some((true, _)) => {}
            Some((false, env)) => {
                return Err(syn::Error::new(
                    env.span(),
                    "JNIEnv must be mutable in trait methods",
                ))
            }
            None => {
                return Err(syn::Error::new(
                    decl.span(),
                    "Trait methods must take a JNIEnv",
                ))
            }
        }

        let args = self
            .args
            .values()
            .filter(|arg| !arg.is_self())
            .enumerate()
            .map(|(i, arg)| {
                let arg_name: TokenStream = if arg.is_env() {
                    "env".to_string().parse()?
                } else {
                    format!("arg_{i}").parse()?
                };

                let arg = arg.java_type.as_interface_arg()?;
                Ok(quote!(#arg_name: #arg, ))
            })
            .collect::<syn::Result<Vec<_>>>()?;

        let vals = self
            .args
            .values()
            .filter(|arg| !arg.is_self())
            .enumerate()
            .filter(|(_, arg)| !arg.is_env())
            .map(|(i, arg)| {
                let in_arg: TokenStream = format!("arg_{i}").parse()?;
                let out_arg: TokenStream = format!("j_arg_{i}").parse()?;
                arg.as_interface_val(in_arg, out_arg)
            })
            .collect::<syn::Result<Vec<_>>>()?;

        let j_args = self
            .args
            .values()
            .filter(|arg| !arg.is_self())
            .enumerate()
            .filter(|(_, arg)| !arg.is_env())
            .map(|(i, _)| -> syn::Result<TokenStream> {
                format!("j_arg_{i}")
                    .parse::<TokenStream>()
                    .map_err(Into::into)
            })
            .collect::<syn::Result<Vec<_>>>()?;

        let args_decl = self
            .args
            .values()
            .filter(|arg| !arg.is_self() && !arg.is_env())
            .map(|arg| arg.java_type.as_jni_declaration())
            .collect::<Vec<_>>()
            .join("");
        let ret_decl = self
            .return_type
            .as_ref()
            .map(|r| r.as_jni_declaration())
            .unwrap_or("V");

        let decl_str = format!("({}){}", args_decl, ret_decl);
        let java_name = self.name.to_case(Case::Camel);

        let ret = decl.sig.output.clone();
        let ret_val = self
            .return_type
            .as_ref()
            .map(|r| r.as_rust_return_val())
            .unwrap_or(quote!(Ok(())));
        Ok(quote! {
            fn #name(&self, #(#args)*) #ret {
                #(#vals)*

                let res = env.call_method(
                    &self.obj,
                    #java_name,
                    #decl_str,
                    &[#(#j_args),*],
                )?;
                #ret_val
            }
        })
    }

    fn get_comment(&self) -> Option<String> {
        self._decl
            .as_ref()
            .map(|decl| {
                decl.attrs()
                    .iter()
                    .filter(|a| a.path().clone().into_token_stream().to_string() == "doc")
                    .filter_map(|a| {
                        if let Meta::NameValue(list) = &a.meta {
                            if let Expr::Lit(s) = &list.value {
                                if let Lit::Str(s) = &s.lit {
                                    return Some(("* ".to_string() + s.value().trim()).to_string());
                                }
                            }
                        }

                        None
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .filter(|s| !s.is_empty())
            .map(|s| format!("/**\n{}\n */\n", s))
    }

    fn from_sig(sig: &Signature, name: String, decl: ImplOrTraitFn) -> syn::Result<Self> {
        let args = sig
            .inputs
            .iter()
            .filter(|arg| !matches!(arg, syn::FnArg::Receiver(_)))
            .map(|arg| Ok((Self::get_name(arg), JavaArg::from_declaration(arg)?)))
            .collect::<syn::Result<IndexMap<_, _>>>()?;
        let return_type = match &sig.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(JavaType::from_declaration(ty)?),
        };
        let self_arg = sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                FnArg::Receiver(r) => {
                    if r.reference.is_some() {
                        Some(Ok(r.mutability.is_some()))
                    } else {
                        Some(Err(syn::Error::new(
                            r.span(),
                            "self argument must be a reference",
                        )))
                    }
                }
                _ => None,
            })
            .next()
            .map_or(Ok(None), |r| r.map(Some))?;

        Ok(Self {
            name,
            original_name: sig.ident.to_string(),
            args,
            return_type,
            static_method: self_arg.is_none(),
            mut_self: self_arg.unwrap_or_default(),
            _decl: Some(decl),
        })
    }
}

impl FromDeclaration<&ImplItemFn, JavaMethod> for JavaMethod {
    fn from_declaration(decl: &ImplItemFn) -> syn::Result<Self> {
        let name = decl
            .get_rename()
            .unwrap_or_else(|| decl.sig.ident.to_string());

        Self::from_sig(&decl.sig, name, ImplOrTraitFn::Impl(decl.clone()))
    }
}

impl FromDeclaration<&TraitItemFn, JavaMethod> for JavaMethod {
    fn from_declaration(decl: &TraitItemFn) -> syn::Result<Self> {
        let name = decl
            .get_rename()
            .unwrap_or_else(|| decl.sig.ident.to_string());

        Self::from_sig(&decl.sig, name, ImplOrTraitFn::Trait(decl.clone()))
    }
}
