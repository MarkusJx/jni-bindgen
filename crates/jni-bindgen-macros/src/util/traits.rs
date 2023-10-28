use crate::util::attrs::BindgenAttrs;
use quote::ToTokens;
use syn::ImplItemFn;

pub trait JniMethod {
    fn get_jni_attr(&self) -> Option<BindgenAttrs>;

    fn get_rename(&self) -> Option<String>;

    fn has_jni(&self) -> bool;

    fn is_constructor(&self) -> bool;
}

impl JniMethod for ImplItemFn {
    fn get_jni_attr(&self) -> Option<BindgenAttrs> {
        self.attrs
            .iter()
            .find(|attr| attr.path().clone().into_token_stream().to_string() == "jni")
            .and_then(|a| a.parse_args::<BindgenAttrs>().ok())
    }

    fn get_rename(&self) -> Option<String> {
        self.get_jni_attr().and_then(|a| a.get_rename())
    }

    fn has_jni(&self) -> bool {
        self.attrs
            .iter()
            .any(|attr| attr.path().clone().into_token_stream().to_string() == "jni")
    }

    fn is_constructor(&self) -> bool {
        self.get_jni_attr()
            .map(|a| a.is_constructor())
            .unwrap_or_default()
    }
}
