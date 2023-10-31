use crate::codegen::traits::FromDeclaration;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use quote::ToTokens;
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{FnArg, PatType, Type, TypePath};

#[derive(Clone)]
pub struct JavaArg {
    pub java_type: JavaType,
    decl: Option<FnArg>,
}

pub enum JNIArgGetter {
    Getter(TokenStream),
    ArgName(String),
}

impl JavaArg {
    pub fn this() -> Self {
        Self {
            java_type: JavaType::This,
            decl: None,
        }
    }

    pub fn as_declaration(&self) -> Option<String> {
        self.java_type.as_declaration()
    }

    pub fn is_self(&self) -> bool {
        if let Some(decl) = &self.decl {
            matches!(decl, FnArg::Receiver(_))
        } else {
            matches!(self.java_type, JavaType::This)
        }
    }

    pub fn as_jni_fn_arg(&self) -> syn::Result<Option<TokenStream>> {
        if self.is_self() {
            return Err(syn::Error::new(
                self.get_span(),
                "Self is not a valid argument for a JNI method",
            ));
        }

        Ok(Some(match &self.java_type {
            JavaType::String => quote!(jni::objects::JString<'local>),
            JavaType::This => {
                return Err(syn::Error::new(
                    self.get_span(),
                    "Self is not a valid argument for a JNI method",
                ))
            }
            JavaType::Env { .. } => return Ok(None),
            JavaType::Void => {
                return Err(syn::Error::new(
                    self.get_span(),
                    "Void is not a valid argument for a JNI method",
                ))
            }
            JavaType::Result(..) => {
                return Err(syn::Error::new(
                    self.get_span(),
                    "Result is not a valid argument for a JNI method",
                ))
            }
            JavaType::Option(..)
            | JavaType::Reference { .. }
            | JavaType::Object
            | JavaType::Vec { .. }
            | JavaType::HashMap { .. } => {
                quote!(jni::objects::JObject<'local>)
            }
            rest => rest.as_jni_return_type(),
        }))
    }

    pub fn as_jni_arg_getter(
        &self,
        arg_name: &str,
        ret_ty: Option<&JavaType>,
    ) -> syn::Result<JNIArgGetter> {
        if self.is_self() {
            return Err(syn::Error::new(
                self.get_span(),
                "Self is not a valid argument for a JNI method",
            ));
        }

        let arg_name = Ident::new(arg_name, self.get_span());
        Ok(JNIArgGetter::Getter(match &self.java_type {
            JavaType::String => {
                ret_ty.unwrap_or(&JavaType::Void).match_error(quote! {
                    env.get_string(&#arg_name)
                        .map_err(|e| e.to_string())
                        .and_then(|s| s.to_str().map_err(|e| e.to_string()).map(|s| s.to_string()))
                })
            },
            JavaType::Env { mutable } => {
                return Ok(JNIArgGetter::ArgName(if *mutable {
                    "&mut env".to_string()
                } else {
                    "&env".to_string()
                }))
            }
            JavaType::Integer => quote!(#arg_name as i32),
            JavaType::Long => quote!(#arg_name as i64),
            JavaType::Boolean => quote!(#arg_name == jni::sys::JNI_TRUE),
            JavaType::Float => quote!(#arg_name as f32),
            JavaType::Double => quote!(#arg_name as f64),
            JavaType::Short => quote!(#arg_name as i16),
            JavaType::Char => quote!(#arg_name as u16),
            JavaType::Byte => quote!(#arg_name as i8),
            JavaType::This => {
                return Err(syn::Error::new(
                    self.get_span(),
                    "Self is not a valid argument for a JNI method",
                ))
            }
            JavaType::Void => quote!(),
            JavaType::Result(_) => {
                return Err(syn::Error::new(
                    self.get_span(),
                    "Result is not a valid argument for a JNI method",
                ))
            }
            JavaType::Option(ty) => {
                ret_ty.unwrap_or(&JavaType::Void).match_error(
                match ty.as_ref() {
                    JavaType::Integer => {
                        quote!(jni_bindgen::conversion::option_convert::i32_from_jni(&mut env, #arg_name))
                    }
                    JavaType::Long => {
                        quote!(jni_bindgen::conversion::option_convert::i64_from_jni(&mut env, #arg_name))
                    }
                    JavaType::Float => {
                        quote!(jni_bindgen::conversion::option_convert::f32_from_jni(&mut env, #arg_name))
                    }
                    JavaType::Double => {
                        quote!(jni_bindgen::conversion::option_convert::f64_from_jni(&mut env, #arg_name))
                    }
                    JavaType::Boolean => {
                        quote!(jni_bindgen::conversion::option_convert::bool_from_jni(&mut env, #arg_name))
                    }
                    JavaType::Short => {
                        quote!(jni_bindgen::conversion::option_convert::i16_from_jni(&mut env, #arg_name))
                    }
                    JavaType::Char => {
                        quote!(jni_bindgen::conversion::option_convert::u16_from_jni(&mut env, #arg_name))
                    }
                    JavaType::Byte => {
                        quote!(jni_bindgen::conversion::option_convert::i8_from_jni(&mut env, #arg_name))
                    }
                    JavaType::String => {
                        quote!(jni_bindgen::conversion::option_convert::string_from_jni(&mut env, #arg_name))
                    },
                    JavaType::Reference {mutable, inner} => {
                        let inner = inner.into_token_stream();

                        let func = if *mutable {
                            quote!(get_struct_mut)
                        } else {
                            quote!(get_struct)
                        };

                        quote! {
                            if #arg_name.is_null() {
                                Ok(None)
                            } else {
                                jni_bindgen::conversion::class_convert::#func::<#inner>(&mut env, #arg_name).map(Some)
                            }
                        }
                    }
                    JavaType::Vec{ty, ..} => {
                        quote! {
                            if #arg_name.is_null() {
                                Ok(None)
                            } else {
                                jni_bindgen::conversion::object_convert::into_vec::<#ty>(&mut env, #arg_name).map(Some)
                            }
                        }
                    }
                    JavaType::HashMap { key, value, .. } => {
                        quote! {
                            if #arg_name.is_null() {
                                Ok(None)
                            } else {
                                jni_bindgen::conversion::object_convert::into_hashmap::<#key, #value>(&mut env, #arg_name).map(Some)
                            }
                        }
                    }
                    JavaType::Wrapped(ty) => {
                        quote!(Option::<jni_bindgen::objects::wrapped::Wrapped<#ty>>::from_jni(&mut env, #arg_name))
                    }
                    _ => return Err(syn::Error::new(self.get_span(), "Unsupported option type")),
                })
            },
            JavaType::Reference { mutable, inner } => {
                let err_ret = ret_ty.unwrap_or(&JavaType::Void).error_return_val();
                let inner = inner.into_token_stream();

                let func = if *mutable {
                    quote!(get_struct_mut)
                } else {
                    quote!(get_struct)
                };

                quote! {
                    if #arg_name.is_null() {
                        let _ = env.throw_new("java/lang/NullPointerException", "The pointer is null");
                        return #err_ret;
                    } else {
                        match jni_bindgen::conversion::class_convert::#func::<#inner>(&mut env, #arg_name) {
                            Ok(ptr) => ptr,
                            Err(e) => {
                                let _ = env.throw_new("java/lang/RuntimeException", e.to_string());
                                return #err_ret;
                            }
                        }
                    }
                }
            },
            JavaType::Wrapped(ty) => {
                quote!(jni_bindgen::objects::wrapped::Wrapped::<#ty>::from_jni(&mut env, #arg_name))
            }
            JavaType::Object => {
                quote!(#arg_name)
            }
            JavaType::Vec{ty, ..} => {
                ret_ty.unwrap_or(&JavaType::Void).match_error(quote!(
                    jni_bindgen::conversion::object_convert::into_vec::<#ty>(&mut env, #arg_name)
                ))
            }
            JavaType::HashMap { key, value, .. } => {
                ret_ty.unwrap_or(&JavaType::Void).match_error(quote!(
                    jni_bindgen::conversion::object_convert::into_hashmap::<#key, #value>(&mut env, #arg_name)
                ))
            }
        }))
    }

    fn get_span(&self) -> Span {
        if let Some(decl) = &self.decl {
            decl.span()
        } else {
            Span::call_site()
        }
    }
}

impl FromDeclaration<&FnArg, JavaArg> for JavaArg {
    fn from_declaration(decl: &FnArg) -> syn::Result<Self> {
        Ok(Self {
            java_type: JavaType::from_declaration(decl)?,
            decl: Some(decl.clone()),
        })
    }
}

#[derive(Clone)]
pub enum JavaType {
    String,
    This,
    Void,
    Integer,
    Long,
    Boolean,
    Float,
    Double,
    Short,
    Char,
    Byte,
    Env {
        mutable: bool,
    },
    Result(Box<JavaType>),
    Option(Box<JavaType>),
    Reference {
        mutable: bool,
        inner: TypePath,
    },
    Wrapped(Type),
    Object,
    Vec {
        ty: Type,
        java_type: Box<JavaType>,
    },
    HashMap {
        key: Type,
        value: Type,
        java_key: Box<JavaType>,
        java_value: Box<JavaType>,
    },
}

impl JavaType {
    pub fn throws(&self) -> Option<String> {
        if let JavaType::Result(_) = self {
            Some(" throws NativeExecutionException".to_string())
        } else {
            None
        }
    }

    pub fn get_imports(&self) -> HashSet<String> {
        let mut imports = HashSet::new();
        match self {
            JavaType::Vec { java_type, .. } => {
                imports.insert("java.util.List".to_string());
                imports.extend(java_type.get_imports());
            }
            JavaType::HashMap {
                java_key,
                java_value,
                ..
            } => {
                imports.insert("java.util.Map".to_string());
                imports.extend(java_key.get_imports());
                imports.extend(java_value.get_imports());
            }
            JavaType::Option(java_type) => {
                imports.extend(java_type.get_imports());
            }
            JavaType::Result(java_type) => {
                imports
                    .insert("com.github.markusjx.jnibindgen.NativeExecutionException".to_string());
                imports.extend(java_type.get_imports());
            }
            _ => {}
        }

        imports
    }

    pub fn as_declaration(&self) -> Option<String> {
        Some(match self {
            JavaType::String => "String".to_string(),
            JavaType::This => "long".to_string(),
            JavaType::Void => "void".to_string(),
            JavaType::Integer => "int".to_string(),
            JavaType::Long => "long".to_string(),
            JavaType::Boolean => "boolean".to_string(),
            JavaType::Float => "float".to_string(),
            JavaType::Double => "double".to_string(),
            JavaType::Short => "short".to_string(),
            JavaType::Char => "char".to_string(),
            JavaType::Byte => "byte".to_string(),
            JavaType::Option(ty) => match ty.as_ref() {
                JavaType::String => "String".to_string(),
                JavaType::Integer => "Integer".to_string(),
                JavaType::Long => "Long".to_string(),
                JavaType::Boolean => "Boolean".to_string(),
                JavaType::Float => "Float".to_string(),
                JavaType::Double => "Double".to_string(),
                JavaType::Short => "Short".to_string(),
                JavaType::Char => "Character".to_string(),
                JavaType::Byte => "Byte".to_string(),
                JavaType::Reference { inner, .. } => inner.into_token_stream().to_string(),
                JavaType::Vec { java_type, .. } => format!("List<{}>", java_type.as_declaration()?),
                JavaType::HashMap {
                    java_key,
                    java_value,
                    ..
                } => format!(
                    "Map<{}, {}>",
                    java_key.as_declaration()?,
                    java_value.as_declaration()?
                ),
                JavaType::Wrapped(ty) => ty.into_token_stream().to_string(),
                _ => panic!(
                    "Unsupported option type: {}",
                    ty.as_declaration().unwrap_or("Env".into())
                ),
            },
            JavaType::Result(ty) => ty.as_declaration()?,
            JavaType::Env { .. } => return None,
            JavaType::Reference { inner, .. } => inner.into_token_stream().to_string(),
            JavaType::Wrapped(ty) => ty.into_token_stream().to_string(),
            JavaType::Object => "Object".to_string(),
            JavaType::Vec { java_type, .. } => format!("List<{}>", java_type.as_declaration()?),
            JavaType::HashMap {
                java_key,
                java_value,
                ..
            } => format!(
                "Map<{}, {}>",
                java_key.as_declaration()?,
                java_value.as_declaration()?
            ),
        })
    }

    pub fn as_jni_return_type(&self) -> TokenStream {
        match self {
            JavaType::String => quote!(jni::sys::jstring),
            JavaType::This => quote!(jni::sys::jlong),
            JavaType::Void => quote!(()),
            JavaType::Integer => quote!(jni::sys::jint),
            JavaType::Long => quote!(jni::sys::jlong),
            JavaType::Boolean => quote!(jni::sys::jboolean),
            JavaType::Float => quote!(jni::sys::jfloat),
            JavaType::Double => quote!(jni::sys::jdouble),
            JavaType::Short => quote!(jni::sys::jshort),
            JavaType::Char => quote!(jni::sys::jchar),
            JavaType::Byte => quote!(jni::sys::jbyte),
            JavaType::Option(..)
            | JavaType::Object
            | JavaType::Vec { .. }
            | JavaType::HashMap { .. } => quote!(jni::sys::jobject),
            JavaType::Result(ty) => ty.as_jni_return_type(),
            JavaType::Env { .. } => panic!("Env is not a valid Java type"),
            JavaType::Reference { .. } => panic!("A reference to a type cannot be returned"),
            JavaType::Wrapped(..) => panic!("Wrapped types cannot be returned"),
        }
    }

    pub fn error_return_val(&self) -> TokenStream {
        match self {
            JavaType::String
            | JavaType::Option(..)
            | JavaType::Object
            | JavaType::Vec { .. }
            | JavaType::HashMap { .. } => {
                quote!(std::ptr::null_mut())
            }
            JavaType::This
            | JavaType::Integer
            | JavaType::Long
            | JavaType::Short
            | JavaType::Char
            | JavaType::Byte => quote!(0),
            JavaType::Void => quote!(),
            JavaType::Boolean => quote!(false),
            JavaType::Float | JavaType::Double => quote!(0.0),
            JavaType::Result(ty) => ty.error_return_val(),
            JavaType::Env { .. } => panic!("Env is not a valid Java type"),
            JavaType::Reference { .. } => panic!("A reference to a type cannot be returned"),
            JavaType::Wrapped(..) => panic!("Wrapped types cannot be returned"),
        }
    }

    pub fn as_jni_return_val(&self) -> TokenStream {
        match self {
            JavaType::String => quote! {
                match env.new_string(res) {
                    Ok(str) => str.into_raw(),
                    Err(e) => {
                        if env.exception_check().unwrap_or_default() {
                            return std::ptr::null_mut();
                        }

                        let _ = env.throw_new("java/lang/RuntimeException", e.to_string());
                        std::ptr::null_mut()
                    }
                }
            },
            JavaType::This => quote!(Box::into_raw(Box::new(res)) as jni::sys::jlong),
            JavaType::Void => quote!(()),
            JavaType::Integer => quote!(res as jni::sys::jint),
            JavaType::Long => quote!(res as jni::sys::jlong),
            JavaType::Boolean => quote! {
                if res {
                    jni::sys::JNI_TRUE
                } else {
                    jni::sys::JNI_FALSE
                }
            },
            JavaType::Float => quote!(res as jni::sys::jfloat),
            JavaType::Double => quote!(res as jni::sys::jdouble),
            JavaType::Short => quote!(res as jni::sys::jshort),
            JavaType::Char => quote!(res as jni::sys::jchar),
            JavaType::Byte => quote!(res as jni::sys::jbyte),
            JavaType::Object => quote!(res.into_raw()),
            JavaType::Result(ty) => {
                let ret = ty.as_jni_return_val();
                let err = ty.error_return_val();
                quote! {
                    match res {
                        Ok(res) => #ret,
                        Err(e) => {
                            if env.exception_check().unwrap_or_default() {
                                return #err;
                            }

                            let _ = env.throw_new("com/github/markusjx/jnibindgen/NativeExecutionException", e.to_string());
                            #err
                        }
                    }
                }
            }
            JavaType::Env { .. } => panic!("Env is not a valid Java type"),
            JavaType::Option(ty) => self.match_error(match ty.as_ref() {
                JavaType::Integer => {
                    quote!(jni_bindgen::conversion::option_convert::i32_into_jni(
                        &mut env, res
                    ))
                }
                JavaType::Long => {
                    quote!(jni_bindgen::conversion::option_convert::i64_into_jni(
                        &mut env, res
                    ))
                }
                JavaType::Float => {
                    quote!(jni_bindgen::conversion::option_convert::f32_into_jni(
                        &mut env, res
                    ))
                }
                JavaType::Double => {
                    quote!(jni_bindgen::conversion::option_convert::f64_into_jni(
                        &mut env, res
                    ))
                }
                JavaType::Boolean => {
                    quote!(jni_bindgen::conversion::option_convert::bool_into_jni(
                        &mut env, res
                    ))
                }
                JavaType::Short => {
                    quote!(jni_bindgen::conversion::option_convert::i16_into_jni(
                        &mut env, res
                    ))
                }
                JavaType::Char => {
                    quote!(jni_bindgen::conversion::option_convert::u16_into_jni(
                        &mut env, res
                    ))
                }
                JavaType::Byte => {
                    quote!(jni_bindgen::conversion::option_convert::i8_into_jni(
                        &mut env, res
                    ))
                }
                JavaType::String => {
                    quote!(jni_bindgen::conversion::option_convert::string_into_jni(
                        &mut env, res
                    ))
                }
                JavaType::Vec { ty, .. } => {
                    quote! {
                        if let Some(s) = res {
                            jni_bindgen::conversion::object_convert::from_vec::<#ty>(&mut env, s)
                        } else {
                            Ok(std::ptr::null_mut())
                        }
                    }
                }
                JavaType::HashMap { key, value, .. } => {
                    quote! {
                        if let Some(s) = res {
                            jni_bindgen::conversion::object_convert::from_hashmap::<#key, #value>(&mut env, s)
                        } else {
                            Ok(std::ptr::null_mut())
                        }
                    }
                }
                _ => panic!("Unsupported option type"),
            }),
            JavaType::Reference { .. } => panic!("A reference to a type cannot be returned"),
            JavaType::Wrapped(..) => panic!("Wrapped types cannot be returned"),
            JavaType::Vec { ty, .. } => {
                self.match_error(quote!(jni_bindgen::conversion::object_convert::from_vec::<#ty>(&mut env, res)))
            }
            JavaType::HashMap { key, value, .. } => {
                self.match_error(quote!(jni_bindgen::conversion::object_convert::from_hashmap::<#key, #value>(&mut env, res)))
            }
        }
    }

    pub fn is_void(&self, check_result: bool) -> bool {
        match self {
            JavaType::Void => true,
            JavaType::Result(r) => check_result && r.is_void(check_result),
            _ => false,
        }
    }

    fn match_error(&self, inner: TokenStream) -> TokenStream {
        let error_return_val = self.error_return_val();

        quote! {
            match #inner {
                Ok(res) => res,
                Err(e) => {
                    if env.exception_check().unwrap_or_default() {
                        return #error_return_val;
                    }

                    let _ = env.throw_new("java/lang/RuntimeException", e.to_string());
                    return #error_return_val;
                }
            }
        }
    }
}

impl FromDeclaration<&FnArg, JavaType> for JavaType {
    fn from_declaration(decl: &FnArg) -> syn::Result<Self> {
        match decl {
            FnArg::Receiver(_) => Ok(JavaType::This),
            FnArg::Typed(PatType { ty, .. }) => Self::from_declaration(ty),
        }
    }
}

impl FromDeclaration<&Box<Type>, JavaType> for JavaType {
    fn from_declaration(decl: &Box<Type>) -> syn::Result<Self> {
        let as_str = decl.into_token_stream().to_string();
        Ok(match as_str.as_str() {
            "String" | "& 'static str" => JavaType::String,
            "Self" => JavaType::This,
            "()" => JavaType::Void,
            "i32" => JavaType::Integer,
            "i64" => JavaType::Long,
            "bool" => JavaType::Boolean,
            "f32" => JavaType::Float,
            "f64" => JavaType::Double,
            "i16" => JavaType::Short,
            "u16" => JavaType::Char,
            "i8" => JavaType::Byte,
            "& jni :: JNIEnv" | "& JNIEnv" | "& mut jni :: JNIEnv" | "& mut JNIEnv" => {
                JavaType::Env {
                    mutable: as_str.starts_with("& mut"),
                }
            }
            _ => {
                match decl.as_ref() {
                    Type::Path(path) => {
                        if let Some(last) = path.path.segments.last() {
                            match last.ident.to_string().as_str() {
                                "Result" | "Option" | "Vec" | "Wrapped" => {
                                    if let syn::PathArguments::AngleBracketed(args) =
                                        &last.arguments
                                    {
                                        if let Some(syn::GenericArgument::Type(ty)) =
                                            args.args.first()
                                        {
                                            match last.ident.to_string().as_str() {
                                                "Result" => {
                                                    return Ok(JavaType::Result(Box::new(
                                                        JavaType::from_declaration(&Box::new(
                                                            ty.clone(),
                                                        ))?,
                                                    )))
                                                }
                                                "Option" => {
                                                    return Ok(JavaType::Option(Box::new(
                                                        JavaType::from_declaration(&Box::new(
                                                            ty.clone(),
                                                        ))?,
                                                    )))
                                                }
                                                "Vec" => {
                                                    return Ok(JavaType::Vec {
                                                        ty: ty.clone(),
                                                        java_type: Box::new(
                                                            JavaType::from_declaration(&Box::new(
                                                                ty.clone(),
                                                            ))?,
                                                        ),
                                                    })
                                                }
                                                "Wrapped" => {
                                                    return Ok(JavaType::Wrapped(ty.clone()))
                                                }
                                                _ => unreachable!(),
                                            }
                                        }
                                    }
                                }
                                "JObject" => return Ok(JavaType::Object),
                                "HashMap" => {
                                    if let syn::PathArguments::AngleBracketed(args) =
                                        &last.arguments
                                    {
                                        if let Some(syn::GenericArgument::Type(ty)) =
                                            args.args.first()
                                        {
                                            if let Some(syn::GenericArgument::Type(ty2)) =
                                                args.args.last()
                                            {
                                                return Ok(JavaType::HashMap {
                                                    key: ty.clone(),
                                                    value: ty2.clone(),
                                                    java_key: Box::new(JavaType::from_declaration(
                                                        &Box::new(ty.clone()),
                                                    )?),
                                                    java_value: Box::new(
                                                        JavaType::from_declaration(&Box::new(
                                                            ty2.clone(),
                                                        ))?,
                                                    ),
                                                });
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Type::Reference(reference) => {
                        if let Type::Path(path) = reference.elem.as_ref() {
                            return Ok(JavaType::Reference {
                                mutable: reference.mutability.is_some(),
                                inner: path.clone(),
                            });
                        }
                    }
                    _ => {}
                }

                Err(syn::Error::new(
                    decl.span(),
                    format!("Unsupported type: '{}'", decl.into_token_stream()),
                ))?
            }
        })
    }
}
