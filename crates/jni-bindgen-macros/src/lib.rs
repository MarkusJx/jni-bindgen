extern crate proc_macro;

use crate::util::expand;
use proc_macro::TokenStream;

mod util;
mod codegen;

#[proc_macro_attribute]
pub fn jni(args: TokenStream, input: TokenStream) -> TokenStream {
    expand::expand(args, input)
        .map_err(|e| e.to_compile_error())
        .unwrap_or_else(|e| e.into())
}
