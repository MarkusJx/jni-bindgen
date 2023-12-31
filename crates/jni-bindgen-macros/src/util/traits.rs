use crate::util::attrs::BindgenAttrs;
use quote::ToTokens;
use syn::{Attribute, Expr, ImplItemFn, Lit, Meta, TraitItemFn};

pub trait JniMethodAttrs {
    fn attrs(&self) -> &Vec<Attribute>;
}

pub trait JniMethod: JniMethodAttrs {
    fn get_jni_attr(&self) -> Option<BindgenAttrs> {
        self.attrs()
            .iter()
            .find(|attr| attr.is_jni())
            .and_then(|a| a.parse_args::<BindgenAttrs>().ok())
    }

    fn get_rename(&self) -> Option<String> {
        self.get_jni_attr().and_then(|a| a.get_rename())
    }

    fn has_jni(&self) -> bool {
        self.attrs()
            .iter()
            .any(|attr| attr.path().clone().into_token_stream().to_string() == "jni")
    }

    fn is_constructor(&self) -> bool {
        self.get_jni_attr()
            .map(|a| a.is_constructor())
            .unwrap_or_default()
    }
}

impl<T: JniMethodAttrs> JniMethod for T {}

impl JniMethodAttrs for ImplItemFn {
    fn attrs(&self) -> &Vec<Attribute> {
        &self.attrs
    }
}

impl JniMethodAttrs for TraitItemFn {
    fn attrs(&self) -> &Vec<Attribute> {
        &self.attrs
    }
}

impl JniMethodAttrs for &Vec<Attribute> {
    fn attrs(&self) -> &Vec<Attribute> {
        self
    }
}

pub trait AnyAttribute {
    fn is_jni(&self) -> bool;
}

impl AnyAttribute for Attribute {
    fn is_jni(&self) -> bool {
        if let Some(last) = self.path().segments.last() {
            last.ident == "jni"
        } else {
            false
        }
    }
}

pub trait GetComment {
    fn get_comment(&self) -> Option<String>;
}

impl GetComment for Vec<Attribute> {
    fn get_comment(&self) -> Option<String> {
        Some(
            self.iter()
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
                .join("\n"),
        )
        .filter(|s| !s.is_empty())
        .map(|s| format!("/**\n{}\n */\n", s))
    }
}
