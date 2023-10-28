use proc_macro2::TokenStream;

pub struct Method {
    pub arg_name: String,
    pub j_arg: TokenStream,
    pub arg_getter: TokenStream,
}

pub trait MethodVec {
    fn java_args(&self) -> TokenStream;
    fn call_args(&self) -> TokenStream;
    fn arg_getters(&self) -> TokenStream;
}

impl MethodVec for Vec<Method> {
    fn java_args(&self) -> TokenStream {
        self.iter()
            .map(|m| m.j_arg.to_string())
            .collect::<Vec<_>>()
            .join(", ")
            .parse()
            .unwrap()
    }

    fn call_args(&self) -> TokenStream {
        self.iter()
            .map(|m| m.arg_name.to_string())
            .collect::<Vec<_>>()
            .join(", ")
            .parse()
            .unwrap()
    }

    fn arg_getters(&self) -> TokenStream {
        self.iter()
            .map(|m| m.arg_getter.to_string())
            .collect::<Vec<_>>()
            .join("\n")
            .parse()
            .unwrap()
    }
}
