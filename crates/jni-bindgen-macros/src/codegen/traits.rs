pub trait AsDeclaration {
    fn as_declaration(&self, definition: bool) -> String;
}

pub trait FromDeclaration<T, R> {
    fn from_declaration(decl: T) -> syn::Result<R>;
}
