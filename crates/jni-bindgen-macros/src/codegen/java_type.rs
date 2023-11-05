use crate::codegen::traits::FromDeclaration;
use crate::util::attrs::BindgenAttrs;
use crate::util::traits::JniMethod;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use quote::ToTokens;
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{FnArg, GenericArgument, PatType, Type, TypeParamBound, TypePath};

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

    pub fn is_env(&self) -> bool {
        matches!(self.java_type, JavaType::Env { .. })
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
            JavaType::Result { .. } => {
                return Err(syn::Error::new(
                    self.get_span(),
                    "Result is not a valid argument for a JNI method",
                ))
            }
            JavaType::Option { .. }
            | JavaType::Reference { .. }
            | JavaType::Object
            | JavaType::Vec { .. }
            | JavaType::HashMap { .. }
            | JavaType::Interface { .. } => {
                quote!(jni::objects::JObject<'local>)
            }
            rest => rest.as_jni_return_type()?,
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
                        .map(|s| s.into())
                        .into_jni_result()
                })?
            },
            JavaType::Env { mutable, .. } => {
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
            JavaType::Result { .. }=> {
                return Err(syn::Error::new(
                    self.get_span(),
                    "Result is not a valid argument for a JNI method",
                ))
            }
            JavaType::Option { java_type, inner } => {
                ret_ty.unwrap_or(&JavaType::Void).match_error(
                match java_type.as_ref() {
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
                    JavaType::Reference {inner} => {
                        let inner = inner.into_token_stream();

                        quote! {
                            if #arg_name.is_null() {
                                Ok(None)
                            } else {
                                <&#inner>::from_jni(&mut env, #arg_name).map(Some)
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
                    _ => return Err(syn::Error::new(inner.span(), "Unsupported option type")),
                })?
            },
            JavaType::Reference { inner } => {
                let err_ret = ret_ty.unwrap_or(&JavaType::Void).error_return_val()?;
                let inner = inner.into_token_stream();

                quote! {
                    if #arg_name.is_null() {
                        let _ = env.throw_new("java/lang/NullPointerException", "The pointer is null");
                        return #err_ret;
                    } else {
                        match <&#inner>::from_jni(&mut env, #arg_name) {
                            Ok(ptr) => ptr,
                            Err(e) => {
                                let _ = env.throw_new("java/lang/RuntimeException", e.to_string());
                                return #err_ret;
                            }
                        }
                    }
                }
            },
            JavaType::Object => {
                quote!(#arg_name)
            }
            JavaType::Vec{ty, ..} => {
                ret_ty.unwrap_or(&JavaType::Void).match_error(quote!(
                    jni_bindgen::conversion::object_convert::into_vec::<#ty>(&mut env, #arg_name)
                ))?
            }
            JavaType::HashMap { key, value, .. } => {
                ret_ty.unwrap_or(&JavaType::Void).match_error(quote!(
                    jni_bindgen::conversion::object_convert::into_hashmap::<#key, #value>(&mut env, #arg_name)
                ))?
            }
            JavaType::Interface { inner, .. } => {
                ret_ty.unwrap_or(&JavaType::Void).match_error(quote!{
                    <Box<#inner>>::from_jni(&mut env, #arg_name)
                })?
            }
        }))
    }

    pub fn as_interface_val(
        &self,
        arg_name: TokenStream,
        out_arg: TokenStream,
    ) -> syn::Result<TokenStream> {
        let inner_arg_name: TokenStream = format!("{}_inner", arg_name).parse()?;
        Ok(match &self.java_type {
            JavaType::String => {
                let inner_arg_name: TokenStream = format!("{}_inner", arg_name).parse()?;
                quote! {
                    let #inner_arg_name = env.new_string(#arg_name)?;
                    let #out_arg = jni::objects::JValue::from(&#inner_arg_name);
                }
            }
            JavaType::This => {
                return Err(syn::Error::new(
                    self.get_span(),
                    "Self is not a valid argument for a JNI method",
                ))
            }
            JavaType::Void => {
                return Err(syn::Error::new(
                    self.get_span(),
                    "Void is not a valid argument for a JNI method",
                ))
            }
            JavaType::Integer
            | JavaType::Long
            | JavaType::Boolean
            | JavaType::Float
            | JavaType::Double
            | JavaType::Short
            | JavaType::Char
            | JavaType::Byte => {
                quote!(let #out_arg = #arg_name.into();)
            }
            JavaType::Env { .. } => {
                return Err(syn::Error::new(
                    self.get_span(),
                    "Env is not a valid argument for a JNI method",
                ))
            }
            JavaType::Result { .. } => {
                return Err(syn::Error::new(
                    self.get_span(),
                    "Result is not a valid argument for a JNI method",
                ))
            }
            JavaType::Option { java_type, inner } => match java_type.as_ref() {
                JavaType::Integer => {
                    quote! {
                        let #inner_arg_name = unsafe {
                            jni::objects::JObject::from_raw(
                                jni_bindgen::conversion::option_convert::i32_into_jni(
                                    env,
                                    #arg_name
                                )?
                            )
                        };
                        let #out_arg = jni::objects::JValue::from(&#inner_arg_name);
                    }
                }
                JavaType::Long => {
                    quote! {
                        let #inner_arg_name = unsafe {
                            jni::objects::JObject::from_raw(
                                jni_bindgen::conversion::option_convert::i64_into_jni(
                                    env,
                                    #arg_name
                                )?
                            )
                        };
                        let #out_arg = jni::objects::JValue::from(&#inner_arg_name);
                    }
                }
                JavaType::Float => {
                    quote! {
                        let #inner_arg_name = unsafe {
                            jni::objects::JObject::from_raw(
                                jni_bindgen::conversion::option_convert::f32_into_jni(
                                    env,
                                    #arg_name
                                )?
                            )
                        };
                        let #out_arg = jni::objects::JValue::from(&#inner_arg_name);
                    }
                }
                JavaType::Double => {
                    quote! {
                        let #inner_arg_name = unsafe {
                            jni::objects::JObject::from_raw(
                                jni_bindgen::conversion::option_convert::f64_into_jni(
                                    env,
                                    #arg_name
                                )?
                            )
                        };
                        let #out_arg = jni::objects::JValue::from(&#inner_arg_name);
                    }
                }
                JavaType::Boolean => {
                    quote! {
                        let #inner_arg_name = unsafe {
                            jni::objects::JObject::from_raw(
                                jni_bindgen::conversion::option_convert::bool_into_jni(
                                    env,
                                    #arg_name
                                )?
                            )
                        };
                        let #out_arg = jni::objects::JValue::from(&#inner_arg_name);
                    }
                }
                JavaType::Short => {
                    quote! {
                        let #inner_arg_name = unsafe {
                            jni::objects::JObject::from_raw(
                                jni_bindgen::conversion::option_convert::i16_into_jni(
                                    env,
                                    #arg_name
                                )?
                            )
                        };
                        let #out_arg = jni::objects::JValue::from(&#inner_arg_name);
                    }
                }
                JavaType::Char => {
                    quote! {
                        let #inner_arg_name = unsafe {
                            jni::objects::JObject::from_raw(
                                jni_bindgen::conversion::option_convert::u16_into_jni(
                                    env,
                                    #arg_name
                                )?
                            )
                        };
                        let #out_arg = jni::objects::JValue::from(&#inner_arg_name);
                    }
                }
                JavaType::Byte => {
                    quote! {
                        let #inner_arg_name = unsafe {
                            jni::objects::JObject::from_raw(
                                jni_bindgen::conversion::option_convert::i8_into_jni(
                                    env,
                                    #arg_name
                                )?
                            )
                        };
                        let #out_arg = jni::objects::JValue::from(&#inner_arg_name);
                    }
                }
                JavaType::String => {
                    quote! {
                        let #inner_arg_name = unsafe {
                            jni::objects::JObject::from_raw(
                                jni_bindgen::conversion::option_convert::string_into_jni(
                                    env,
                                    #arg_name
                                )?
                            )
                        };
                        let #out_arg = jni::objects::JValue::from(&#inner_arg_name);
                    }
                }
                JavaType::Vec { ty, .. } => {
                    quote! {
                        let #inner_arg_name = if let Some(s) = #arg_name {
                            unsafe {
                                jni::objects::JObject::from_raw(
                                    jni_bindgen::conversion::object_convert::from_vec::<#ty>(env, s)?
                                )
                            }
                        } else {
                            jni::objects::JObject::null()
                        };
                        let #out_arg = jni::objects::JValue::from(&#inner_arg_name);
                    }
                }
                JavaType::HashMap { key, value, .. } => {
                    quote! {
                        let #inner_arg_name = if let Some(s) = #arg_name {
                            unsafe {
                                jni::objects::JObject::from_raw(
                                    jni_bindgen::conversion::object_convert::from_hashmap::<#key, #value>(env, s)?
                                )
                            }
                        } else {
                            jni::objects::JObject::null()
                        };
                        let #out_arg = jni::objects::JValue::from(&#inner_arg_name);
                    }
                }
                _ => return Err(syn::Error::new(inner.span(), "Unsupported option type")),
            },
            JavaType::Reference { .. } => {
                return Err(syn::Error::new(
                    self.get_span(),
                    "A reference to a type cannot be passed",
                ))
            }
            JavaType::Object => {
                quote!(let #out_arg = (&#arg_name).into();)
            }
            JavaType::Vec { ty, .. } => {
                quote! {
                    let #inner_arg_name = unsafe {
                        jni::objects::JObject::from_raw(jni_bindgen::conversion::object_convert::from_vec::<#ty>(env, #arg_name)?)
                    };
                    let #out_arg = jni::objects::JValue::from(&#inner_arg_name);
                }
            }
            JavaType::HashMap { key, value, .. } => {
                quote! {
                    let #inner_arg_name = unsafe {
                        jni::objects::JObject::from_raw(jni_bindgen::conversion::object_convert::from_hashmap::<#key, #value>(env, #arg_name)?)
                    };
                    let #out_arg = jni::objects::JValue::from(&#inner_arg_name);
                }
            }
            JavaType::Interface { .. } => {
                quote!(let #out_arg = #arg_name.obj.into();)
            }
        })
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
        inner: Type,
    },
    Result {
        java_type: Box<JavaType>,
        result_type: TypePath,
    },
    Option {
        java_type: Box<JavaType>,
        inner: Type,
    },
    Reference {
        inner: TypePath,
    },
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
    Interface {
        inner: Type,
        class_name: String,
    },
}

impl JavaType {
    pub fn throws(&self) -> Option<String> {
        if let JavaType::Result { .. } = self {
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
            JavaType::Option { java_type, .. } => {
                imports.extend(java_type.get_imports());
            }
            JavaType::Result { java_type, .. } => {
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
            JavaType::Option { java_type, .. } => match java_type.as_ref() {
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
                _ => panic!(
                    "Unsupported option type: {}",
                    java_type.as_declaration().unwrap_or("Env".into())
                ),
            },
            JavaType::Result { java_type, .. } => java_type.as_declaration()?,
            JavaType::Env { .. } => return None,
            JavaType::Reference { inner, .. } => inner.into_token_stream().to_string(),
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
            JavaType::Interface { class_name, .. } => class_name.clone(),
        })
    }

    pub fn as_jni_return_type(&self) -> syn::Result<TokenStream> {
        Ok(match self {
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
            JavaType::Option { .. }
            | JavaType::Object
            | JavaType::Vec { .. }
            | JavaType::HashMap { .. } => quote!(jni::sys::jobject),
            JavaType::Result { java_type, .. } => java_type.as_jni_return_type()?,
            JavaType::Env { inner, .. } => {
                return Err(syn::Error::new(
                    inner.span(),
                    "Env is not a valid Java type",
                ))
            }
            JavaType::Reference { inner, .. } => {
                return Err(syn::Error::new(
                    inner.span(),
                    "A reference to a type cannot be returned",
                ))
            }
            JavaType::Interface { inner, .. } => {
                return Err(syn::Error::new(
                    inner.span(),
                    "Interfaces cannot be returned",
                ))
            }
        })
    }

    pub fn error_return_val(&self) -> syn::Result<TokenStream> {
        Ok(match self {
            JavaType::String
            | JavaType::Option { .. }
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
            JavaType::Boolean => quote!(jni::sys::JNI_FALSE),
            JavaType::Float | JavaType::Double => quote!(0.0),
            JavaType::Result { java_type, .. } => java_type.error_return_val()?,
            JavaType::Env { inner, .. } => {
                return Err(syn::Error::new(
                    inner.span(),
                    "Env is not a valid Java type",
                ))
            }
            JavaType::Reference { inner, .. } => {
                return Err(syn::Error::new(
                    inner.span(),
                    "A reference to a type cannot be returned",
                ))
            }
            JavaType::Interface { inner, .. } => {
                return Err(syn::Error::new(
                    inner.span(),
                    "Interfaces cannot be returned",
                ))
            }
        })
    }

    pub fn as_interface_arg(&self) -> syn::Result<TokenStream> {
        Ok(match self {
            JavaType::String => quote!(String),
            JavaType::This => quote!(&self),
            JavaType::Void => quote!(()),
            JavaType::Integer => quote!(i32),
            JavaType::Long => quote!(i64),
            JavaType::Boolean => quote!(bool),
            JavaType::Float => quote!(f32),
            JavaType::Double => quote!(f64),
            JavaType::Short => quote!(i16),
            JavaType::Char => quote!(u16),
            JavaType::Byte => quote!(i8),
            JavaType::Env { .. } => quote!(&mut jni::JNIEnv),
            JavaType::Result { result_type, .. } => quote!(#result_type),
            JavaType::Option { java_type, .. } => {
                let inner_ty = java_type.as_interface_arg()?;
                quote!(Option<#inner_ty>)
            }
            JavaType::Reference { inner } => quote!(&#inner),
            JavaType::Object => quote!(jni::objects::JObject),
            JavaType::Vec { java_type, .. } => {
                let inner_ty = java_type.as_interface_arg()?;
                quote!(Vec<#inner_ty>)
            }
            JavaType::HashMap {
                java_key,
                java_value,
                ..
            } => {
                let key_ty = java_key.as_interface_arg()?;
                let value_ty = java_value.as_interface_arg()?;
                quote!(HashMap<#key_ty, #value_ty>)
            }
            JavaType::Interface { inner, .. } => quote!(Box<#inner>),
        })
    }

    pub fn as_jni_declaration(&self) -> String {
        match self {
            JavaType::String => "Ljava/lang/String;".to_string(),
            JavaType::This => panic!("Self is not a valid argument for a JNI method"),
            JavaType::Void => "V".to_string(),
            JavaType::Integer => "I".to_string(),
            JavaType::Long => "J".to_string(),
            JavaType::Boolean => "Z".to_string(),
            JavaType::Float => "F".to_string(),
            JavaType::Double => "D".to_string(),
            JavaType::Short => "S".to_string(),
            JavaType::Char => "C".to_string(),
            JavaType::Byte => "B".to_string(),
            JavaType::Env { .. } => panic!("Env is not a valid argument for a JNI method"),
            JavaType::Result { java_type, .. } => java_type.as_jni_declaration(),
            JavaType::Option { java_type, .. } => match java_type.as_ref() {
                JavaType::String => "Ljava/lang/String;".to_string(),
                JavaType::This => panic!("Self is not a valid argument for a JNI method"),
                JavaType::Void => "V".to_string(),
                JavaType::Integer => "Ljava/lang/Integer;".to_string(),
                JavaType::Long => "Ljava/lang/Long;".to_string(),
                JavaType::Boolean => "Ljava/lang/Boolean;".to_string(),
                JavaType::Float => "Ljava/lang/Float;".to_string(),
                JavaType::Double => "Ljava/lang/Double;".to_string(),
                JavaType::Short => "Ljava/lang/Short;".to_string(),
                JavaType::Char => "Ljava/lang/Character;".to_string(),
                JavaType::Byte => "Ljava/lang/Byte;".to_string(),
                JavaType::Env { .. } => panic!("Env is not a valid argument for a JNI method"),
                JavaType::Result { java_type, .. } => java_type.as_jni_declaration(),
                JavaType::Option { .. } => {
                    panic!("Option is not a valid argument for a JNI method")
                }
                JavaType::Reference { .. } => panic!("A reference to a type cannot be passed"),
                JavaType::Object => "Ljava/lang/Object;".to_string(),
                JavaType::Vec { .. } => "Ljava/util/List;".to_string(),
                JavaType::HashMap { .. } => "Ljava/util/Map;".to_string(),
                JavaType::Interface { class_name, .. } => {
                    format!("L{};", class_name.replace('.', "/"))
                }
            },
            JavaType::Reference { .. } => panic!("A reference to a type cannot be passed"),
            JavaType::Object => "Ljava/lang/Object;".to_string(),
            JavaType::Vec { .. } => "Ljava/util/List;".to_string(),
            JavaType::HashMap { .. } => "Ljava/util/Map;".to_string(),
            JavaType::Interface { class_name, .. } => {
                format!("L{};", class_name.replace('.', "/"))
            }
        }
    }

    pub fn as_rust_return_val(&self) -> TokenStream {
        match self {
            JavaType::String => quote! {
                env.get_string(&jni::objects::JString::from(res.l()?))
                    .map(Into::into)
                    .map_err(Into::into)
            },
            JavaType::This => panic!("Self is not a valid argument for a JNI method"),
            JavaType::Void => quote!(Ok(())),
            JavaType::Integer => quote!(res.i().map_err(Into::into)),
            JavaType::Long => quote!(res.j().map_err(Into::into)),
            JavaType::Boolean => quote!(res.z().map_err(Into::into)),
            JavaType::Float => quote!(res.f().map_err(Into::into)),
            JavaType::Double => quote!(res.d().map_err(Into::into)),
            JavaType::Short => quote!(res.s().map_err(Into::into)),
            JavaType::Char => quote!(res.c().map_err(Into::into)),
            JavaType::Byte => quote!(res.b().map_err(Into::into)),
            JavaType::Env { .. } => panic!("Env is not a valid argument for a JNI method"),
            JavaType::Result { java_type, .. } => java_type.as_rust_return_val(),
            JavaType::Option { java_type, .. } => {
                match java_type.as_ref() {
                    JavaType::String => {
                        quote!(jni_bindgen::conversion::option_convert::string_from_jni(
                            env,
                            res.l()?
                        )
                        .map_err(Into::into))
                    }
                    JavaType::This => panic!("Self is not a valid argument for a JNI method"),
                    JavaType::Void => quote!(Ok(None)),
                    JavaType::Integer => quote!(
                        jni_bindgen::conversion::option_convert::i32_from_jni(env, res.l()?)
                            .map_err(Into::into)
                    ),
                    JavaType::Long => quote!(
                        jni_bindgen::conversion::option_convert::i64_from_jni(env, res.l()?)
                            .map_err(Into::into)
                    ),
                    JavaType::Boolean => quote!(
                        jni_bindgen::conversion::option_convert::bool_from_jni(env, res.l()?)
                            .map_err(Into::into)
                    ),
                    JavaType::Float => quote!(
                        jni_bindgen::conversion::option_convert::f32_from_jni(env, res.l()?)
                            .map_err(Into::into)
                    ),
                    JavaType::Double => quote!(
                        jni_bindgen::conversion::option_convert::f64_from_jni(env, res.l()?)
                            .map_err(Into::into)
                    ),
                    JavaType::Short => quote!(
                        jni_bindgen::conversion::option_convert::i16_from_jni(env, res.l()?)
                            .map_err(Into::into)
                    ),
                    JavaType::Char => quote!(
                        jni_bindgen::conversion::option_convert::u16_from_jni(env, res.l()?)
                            .map_err(Into::into)
                    ),
                    JavaType::Byte => quote!(jni_bindgen::conversion::option_convert::i8_from_jni(
                        env,
                        res.l()?
                    )
                    .map_err(Into::into)),
                    JavaType::Env { .. } => panic!("Env is not a valid argument for a JNI method"),
                    JavaType::Result { .. } => {
                        panic!("Result is not a valid argument for a JNI method")
                    }
                    JavaType::Option { .. } => panic!("Option must not be nested"),
                    JavaType::Reference { inner } => {
                        let inner = inner.into_token_stream();

                        quote! {
                            let res_unpacked = res.l()?;
                            if res_unpacked.is_null() {
                                Ok(None)
                            } else {
                                <&#inner>::from_jni(env, res_unpacked).map(Some).map_err(Into::into)
                            }
                        }
                    }
                    JavaType::Object => {
                        quote! {
                            let res_unpacked = res.l()?;
                            if res_unpacked.is_null() {
                                Ok(None)
                            } else {
                                Ok(Some(res_unpacked.into()))
                            }
                        }
                    }
                    JavaType::Vec { ty, .. } => {
                        quote! {
                            let res_unpacked = res.l()?;
                            if res_unpacked.is_null() {
                                Ok(None)
                            } else {
                                jni_bindgen::conversion::object_convert::into_vec::<#ty>(env, res_unpacked)
                                    .map(Some)
                                    .map_err(Into::into)
                            }
                        }
                    }
                    JavaType::HashMap { key, value, .. } => {
                        quote! {
                            let res_unpacked = res.l()?;
                            if res_unpacked.is_null() {
                                Ok(None)
                            } else {
                                jni_bindgen::conversion::object_convert::into_hashmap::<#key, #value>(env, res_unpacked)
                                    .map(Some)
                                    .map_err(Into::into)
                            }
                        }
                    }
                    JavaType::Interface { inner, .. } => {
                        quote! {
                            let res_unpacked = res.l()?;
                            if res_unpacked.is_null() {
                                Ok(None)
                            } else {
                                <Box<#inner>>::from_jni(env, res_unpacked).map(Some).map_err(Into::into)
                            }
                        }
                    }
                }
            }
            JavaType::Reference { .. } => panic!("A reference to a type cannot be passed"),
            JavaType::Object => quote!(res.l().map_err(Into::into)),
            JavaType::Vec { ty, .. } => {
                quote!(jni_bindgen::conversion::object_convert::into_vec::<#ty>(env, res.l()?).map_err(Into::into))
            }
            JavaType::HashMap { key, value, .. } => {
                quote!(jni_bindgen::conversion::object_convert::into_hashmap::<#key, #value>(env, res.l()?).map_err(Into::into))
            }
            JavaType::Interface { inner, .. } => {
                quote!(Box<#inner>::from_jni(env, res.l()?).map_err(Into::into))
            }
        }
    }

    pub fn as_jni_return_val(&self) -> syn::Result<TokenStream> {
        Ok(match self {
            JavaType::String => quote! {
                match env.new_string(res).into_jni_result() {
                    Ok(str) => str.into_raw(),
                    Err(e) => {
                        if env.exception_check().unwrap_or_default() {
                            return std::ptr::null_mut();
                        }

                        e.throw(&mut env);
                        std::ptr::null_mut()
                    }
                }
            },
            JavaType::This => quote!(
                Box::into_raw(Box::new(res)) as jni::sys::jlong
            ),
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
            JavaType::Result {java_type, ..} => {
                let ret = java_type.as_jni_return_val()?;
                let err = java_type.error_return_val()?;
                quote! {
                    match res.into_jni_result()
                        .map_err(|e| e.or_class(jni_bindgen::errors::jni_error::ErrorClass::NativeExecutionException))
                    {
                        Ok(res) => #ret,
                        Err(e) => {
                            if env.exception_check().unwrap_or_default() {
                                return #err;
                            }

                            e.throw(&mut env);
                            #err
                        }
                    }
                }
            }
            JavaType::Env { inner, .. } => return Err(syn::Error::new(
                inner.span(),
                "Env is not a valid Java type"
            )),
            JavaType::Option { java_type, inner } => self.match_error(match java_type.as_ref() {
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
                _ => return Err(syn::Error::new(inner.span(), "Unsupported option type")),
            })?,
            JavaType::Reference { inner, .. } => return Err(syn::Error::new(
                inner.span(),
                "A reference to a type cannot be returned"
            )),
            JavaType::Vec { ty, .. } => {
                self.match_error(quote!(jni_bindgen::conversion::object_convert::from_vec::<#ty>(&mut env, res)))?
            }
            JavaType::HashMap { key, value, .. } => {
                self.match_error(quote!(jni_bindgen::conversion::object_convert::from_hashmap::<#key, #value>(&mut env, res)))?
            }
            JavaType::Interface { inner, .. } => return Err(syn::Error::new(
                inner.span(),
                "Interfaces cannot be returned"
            ))
        })
    }

    pub fn is_void(&self, check_result: bool) -> bool {
        match self {
            JavaType::Void => true,
            JavaType::Result { java_type, .. } => check_result && java_type.is_void(check_result),
            _ => false,
        }
    }

    fn match_error(&self, inner: TokenStream) -> syn::Result<TokenStream> {
        let error_return_val = self.error_return_val()?;

        Ok(quote! {
            match #inner.into_jni_result() {
                Ok(res) => res,
                Err(e) => {
                    if env.exception_check().unwrap_or_default() {
                        return #error_return_val;
                    }

                    e.throw(&mut env);
                    return #error_return_val;
                }
            }
        })
    }

    fn match_typed<T: Spanned>(
        ty: &Type,
        decl: &T,
        attr: Option<BindgenAttrs>,
    ) -> syn::Result<Option<Self>> {
        match ty {
            Type::Path(path) => {
                if let Some(last) = path.path.segments.last() {
                    match last.ident.to_string().as_str() {
                        "Result" | "Option" | "Vec" | "Box" => {
                            if let syn::PathArguments::AngleBracketed(args) = &last.arguments {
                                if let Some(GenericArgument::Type(ty)) = args.args.first() {
                                    match last.ident.to_string().as_str() {
                                        "Result" => {
                                            return Ok(Some(JavaType::Result {
                                                result_type: path.clone(),
                                                java_type: Box::new(JavaType::from_declaration(
                                                    &Box::new(ty.clone()),
                                                )?),
                                            }))
                                        }
                                        "Option" => {
                                            return Ok(Some(JavaType::Option {
                                                java_type: Box::new(JavaType::from_declaration(
                                                    &Box::new(ty.clone()),
                                                )?),
                                                inner: ty.clone(),
                                            }))
                                        }
                                        "Vec" => {
                                            return Ok(Some(JavaType::Vec {
                                                ty: ty.clone(),
                                                java_type: Box::new(JavaType::from_declaration(
                                                    &Box::new(ty.clone()),
                                                )?),
                                            }))
                                        }
                                        "Box" => {
                                            let mut ty = ty.clone();
                                            let mut class_name;

                                            if let Type::TraitObject(obj) = &mut ty {
                                                class_name = obj
                                                    .bounds
                                                    .iter()
                                                    .find_map(|b| match b {
                                                        TypeParamBound::Trait(t) => Some(t),
                                                        _ => None,
                                                    })
                                                    .map(|t| {
                                                        t.path
                                                            .segments
                                                            .iter()
                                                            .filter(|s| s.ident != "crate")
                                                            .map(|s| s.ident.to_string())
                                                            .collect::<Vec<_>>()
                                                            .join(".")
                                                    });

                                                if let Some(l) =
                                                    obj.bounds.iter_mut().find_map(|b| match b {
                                                        TypeParamBound::Lifetime(l) => Some(l),
                                                        _ => None,
                                                    })
                                                {
                                                    l.ident = Ident::new("local", l.ident.span());
                                                } else {
                                                    return Err(syn::Error::new(
                                                        decl.span(),
                                                        "Box must have a lifetime parameter",
                                                    ));
                                                }
                                            } else {
                                                return Err(syn::Error::new(
                                                    decl.span(),
                                                    "Box must have a dyn Trait parameter",
                                                ));
                                            }

                                            if !args
                                                .args
                                                .iter()
                                                .any(|a| matches!(a, GenericArgument::Type(_)))
                                            {
                                                return Err(syn::Error::new(
                                                    decl.span(),
                                                    "Box must have a lifetime parameter",
                                                ));
                                            }

                                            if class_name.is_none() {
                                                let Some(attr) = attr.as_ref() else {
                                                    return Err(syn::Error::new(
                                                        decl.span(),
                                                        "Box must have a #[jni(class_name = \"...\")] attribute",
                                                    ));
                                                };

                                                class_name = Some(
                                                    attr
                                                        .get_class_name()
                                                        .ok_or(syn::Error::new(
                                                            decl.span(),
                                                            "jni attribute must have a class_name member",
                                                        ))?
                                                );
                                            } else if let Some(attr) = attr.as_ref() {
                                                if let Some(class_name_attr) = attr.get_class_name()
                                                {
                                                    class_name = Some(class_name_attr);
                                                }
                                            }

                                            return Ok(Some(JavaType::Interface {
                                                inner: ty.clone(),
                                                class_name: class_name.unwrap(),
                                            }));
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                            }
                        }
                        "JObject" => return Ok(Some(JavaType::Object)),
                        "HashMap" => {
                            if let syn::PathArguments::AngleBracketed(args) = &last.arguments {
                                if let Some(GenericArgument::Type(ty)) = args.args.first() {
                                    if let Some(GenericArgument::Type(ty2)) = args.args.last() {
                                        return Ok(Some(JavaType::HashMap {
                                            key: ty.clone(),
                                            value: ty2.clone(),
                                            java_key: Box::new(JavaType::from_declaration(
                                                &Box::new(ty.clone()),
                                            )?),
                                            java_value: Box::new(JavaType::from_declaration(
                                                &Box::new(ty2.clone()),
                                            )?),
                                        }));
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
                    if let Some(last) = path.path.segments.last() {
                        if last.ident == "JNIEnv" {
                            return Ok(Some(JavaType::Env {
                                mutable: reference.mutability.is_some(),
                                inner: ty.clone(),
                            }));
                        }
                    }

                    return if reference.mutability.is_some() {
                        Err(syn::Error::new(
                            decl.span(),
                            "Mutable references are not supported",
                        ))
                    } else {
                        Ok(Some(JavaType::Reference {
                            inner: path.clone(),
                        }))
                    };
                }
            }
            _ => {}
        }

        Ok(None)
    }
}

impl FromDeclaration<&FnArg, JavaType> for JavaType {
    fn from_declaration(decl: &FnArg) -> syn::Result<Self> {
        match decl {
            FnArg::Receiver(_) => Ok(JavaType::This),
            FnArg::Typed(PatType { ty, attrs, .. }) => {
                let attr = attrs.get_jni_attr();

                if let Some(res) = Self::match_typed(ty, decl, attr)? {
                    Ok(res)
                } else {
                    Self::from_declaration(ty)
                }
            }
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
                    inner: decl.as_ref().clone(),
                }
            }
            _ => {
                if let Some(res) = Self::match_typed(decl, decl, None)? {
                    res
                } else {
                    Err(syn::Error::new(
                        decl.span(),
                        format!("Unsupported type: '{}'", decl.into_token_stream()),
                    ))?
                }
            }
        })
    }
}
