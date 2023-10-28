use crate::codegen::traits::FromDeclaration;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{FnArg, PatType, Type};

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
            rest => rest.as_jni_return_type(),
        }))
    }

    pub fn as_jni_arg_getter(&self, arg_name: &str) -> syn::Result<JNIArgGetter> {
        if self.is_self() {
            return Err(syn::Error::new(
                self.get_span(),
                "Self is not a valid argument for a JNI method",
            ));
        }

        let arg_name = Ident::new(arg_name, self.get_span());
        Ok(JNIArgGetter::Getter(match &self.java_type {
            JavaType::String => quote! {
                env.get_string(&#arg_name)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
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
    Env { mutable: bool },
    Result(Box<JavaType>),
}

impl JavaType {
    pub fn throws(&self) -> Option<String> {
        if let JavaType::Result(_) = self {
            Some(" throws NativeExecutionException".to_string())
        } else {
            None
        }
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
            JavaType::Result(ty) => ty.as_declaration()?,
            JavaType::Env { .. } => return None,
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
            JavaType::Result(ty) => ty.as_jni_return_type(),
            JavaType::Env { .. } => panic!("Env is not a valid Java type"),
        }
    }

    pub fn error_return_val(&self) -> TokenStream {
        match self {
            JavaType::String => quote!(std::ptr::null_mut()),
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
        }
    }

    pub fn as_jni_return_val(&self) -> TokenStream {
        match self {
            JavaType::String => quote! {
                match env.new_string(res) {
                    Ok(str) => str.into_raw(),
                    Err(e) => {
                        env.throw_new("java/lang/RuntimeException", e.to_string())
                            .unwrap();
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
            JavaType::Result(ty) => {
                let ret = ty.as_jni_return_val();
                let err = ty.error_return_val();
                quote! {
                    match res {
                        Ok(res) => #ret,
                        Err(e) => {
                            env.throw_new("com/github/markusjx/rust/NativeExecutionException", e.to_string())
                                .unwrap();
                            #err
                        }
                    }
                }
            }
            JavaType::Env { .. } => panic!("Env is not a valid Java type"),
        }
    }

    pub fn is_void(&self, check_result: bool) -> bool {
        match self {
            JavaType::Void => true,
            JavaType::Result(r) => check_result && r.is_void(check_result),
            _ => false,
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
                if let Type::Path(path) = decl.as_ref() {
                    if let Some(last) = path.path.segments.last() {
                        if last.ident == "Result" {
                            if let syn::PathArguments::AngleBracketed(args) = &last.arguments {
                                if let Some(syn::GenericArgument::Type(ty)) = args.args.first() {
                                    return Ok(JavaType::Result(Box::new(
                                        JavaType::from_declaration(&Box::new(ty.clone()))?,
                                    )));
                                }
                            }
                        }
                    }
                }

                Err(syn::Error::new(
                    decl.span(),
                    format!("Unsupported type {}", decl.into_token_stream()),
                ))?
            }
        })
    }
}
