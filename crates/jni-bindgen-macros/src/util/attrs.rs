use proc_macro2::{Ident, Span};
use std::cell::Cell;
use syn::parse::{Parse, ParseStream, Result as SynResult};
use syn::Token;

#[derive(Debug, Clone)]
pub struct BindgenAttrs {
    /// Whether `#[jni]` attribute exists
    pub exists: bool,
    /// List of parsed attributes
    pub attrs: Vec<(Cell<bool>, BindgenAttr)>,
    /// Span of original attribute
    pub span: Span,
}

impl BindgenAttrs {
    fn get_attr<F: FnMut(&(Cell<bool>, BindgenAttr)) -> Option<R>, R>(&self, f: F) -> Option<R> {
        self.attrs.iter().filter_map(f).next()
    }

    pub fn get_class_name(&self) -> Option<String> {
        self.get_attr(|attr| match &attr.1 {
            BindgenAttr::ClassName(_, name, _) => Some(name.clone()),
            _ => None,
        })
    }

    pub fn get_namespace(&self) -> syn::Result<String> {
        self.get_attr(|e| match &e.1 {
            BindgenAttr::Namespace(_, name, _) => Some(name.clone()),
            _ => None,
        })
        .ok_or(syn::Error::new(
            self.span,
            "Missing namespace = \"...\" attribute",
        ))
    }

    pub fn get_rename(&self) -> Option<String> {
        self.get_attr(|arg| match &arg.1 {
            BindgenAttr::Rename(_, name, _) => Some(name.clone()),
            _ => None,
        })
    }

    pub fn is_constructor(&self) -> bool {
        self.attrs
            .iter()
            .any(|arg| matches!(&arg.1, BindgenAttr::Constructor(_)))
    }

    pub fn load_lib(&self) -> Option<String> {
        self.get_attr(|arg| match &arg.1 {
            BindgenAttr::LoadLib(_, name, _) => Some(name.clone()),
            _ => None,
        })
    }
}

impl Default for BindgenAttrs {
    fn default() -> BindgenAttrs {
        // Add 1 to the list of parsed attribute sets. We'll use this counter to
        // sanity check that we call `check_used` an appropriate number of
        // times.
        //ATTRS.with(|state| state.parsed.set(state.parsed.get() + 1));
        BindgenAttrs {
            span: Span::call_site(),
            attrs: Vec::new(),
            exists: false,
        }
    }
}

impl Parse for BindgenAttrs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let mut attrs = BindgenAttrs::default();
        if input.is_empty() {
            return Ok(attrs);
        }

        let opts = syn::punctuated::Punctuated::<_, syn::token::Comma>::parse_terminated(input)?;
        attrs.attrs = opts.into_iter().map(|c| (Cell::new(false), c)).collect();
        Ok(attrs)
    }
}

pub struct AnyIdent(Ident);

impl Parse for AnyIdent {
    fn parse(input: ParseStream) -> SynResult<Self> {
        input.step(|cursor| match cursor.ident() {
            Some((ident, remaining)) => Ok((AnyIdent(ident), remaining)),
            None => Err(cursor.error("expected an identifier")),
        })
    }
}

macro_rules! attrgen {
    ($mac:ident) => {
        $mac! {
            (package, Namespace(Span, String, Span)),
            (rename, Rename(Span, String, Span)),
            (constructor, Constructor(Span)),
            (load_lib, LoadLib(Span, String, Span)),
            (class_name, ClassName(Span, String, Span)),
        }
    };
}

macro_rules! gen_bindgen_attr {
  ($( ($method:ident, $($variants:tt)*) ,)*) => {
    /// The possible attributes in the `#[napi]`.
    #[derive(Debug, Clone)]
    pub enum BindgenAttr {
      $($($variants)*,)*
    }
  }
}

impl Parse for BindgenAttr {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let original = input.fork();
        let attr: AnyIdent = input.parse()?;
        let attr = attr.0;
        let attr_span = attr.span();
        let attr_string = attr.to_string();
        let raw_attr_string = format!("r#{}", attr_string);

        macro_rules! parsers {
      ($(($name:ident, $($contents:tt)*),)*) => {
        $(
          if attr_string == stringify!($name) || raw_attr_string == stringify!($name) {
            parsers!(
              @parser
              $($contents)*
            );
          }
        )*
      };

      (@parser $variant:ident(Span)) => ({
        return Ok(BindgenAttr::$variant(attr_span));
      });

      (@parser $variant:ident(Span, Ident)) => ({
        input.parse::<Token![=]>()?;
        let ident = input.parse::<AnyIdent>()?.0;
        return Ok(BindgenAttr::$variant(attr_span, ident))
      });

      (@parser $variant:ident(Span, Option<Ident>)) => ({
        if input.parse::<Token![=]>().is_ok() {
          let ident = input.parse::<AnyIdent>()?.0;
          return Ok(BindgenAttr::$variant(attr_span, Some(ident)))
        } else {
          return Ok(BindgenAttr::$variant(attr_span, None));
        }
      });

        (@parser $variant:ident(Span, syn::Path)) => ({
            input.parse::<Token![=]>()?;
            return Ok(BindgenAttr::$variant(attr_span, input.parse()?));
        });

        (@parser $variant:ident(Span, syn::Expr)) => ({
            input.parse::<Token![=]>()?;
            return Ok(BindgenAttr::$variant(attr_span, input.parse()?));
        });

        (@parser $variant:ident(Span, String, Span)) => ({
          input.parse::<Token![=]>()?;
          let (val, span) = match input.parse::<syn::LitStr>() {
            Ok(str) => (str.value(), str.span()),
            Err(_) => {
              let ident = input.parse::<AnyIdent>()?.0;
              (ident.to_string(), ident.span())
            }
          };
          return Ok(BindgenAttr::$variant(attr_span, val, span))
        });

        (@parser $variant:ident(Span, Option<bool>)) => ({
          if let Ok(_) = input.parse::<Token![=]>() {
            let (val, _) = match input.parse::<syn::LitBool>() {
              Ok(str) => (str.value(), str.span()),
              Err(_) => {
                let ident = input.parse::<AnyIdent>()?.0;
                (true, ident.span())
              }
            };
            return Ok::<BindgenAttr, syn::Error>(BindgenAttr::$variant(attr_span, Some(val)))
          } else {
            return Ok(BindgenAttr::$variant(attr_span, Some(true)))
          }
        });

        (@parser $variant:ident(Span, Vec<String>, Vec<Span>)) => ({
          input.parse::<Token![=]>()?;
          let (vals, spans) = match input.parse::<syn::ExprArray>() {
            Ok(exprs) => {
              let mut vals = vec![];
              let mut spans = vec![];

              for expr in exprs.elems.iter() {
                if let syn::Expr::Lit(syn::ExprLit {
                  lit: syn::Lit::Str(ref str),
                  ..
                }) = expr {
                  vals.push(str.value());
                  spans.push(str.span());
                } else {
                  return Err(syn::Error::new(expr.span(), "expected string literals"));
                }
              }

              (vals, spans)
            },
            Err(_) => {
              let ident = input.parse::<AnyIdent>()?.0;
              (vec![ident.to_string()], vec![ident.span()])
            }
          };
          return Ok(BindgenAttr::$variant(attr_span, vals, spans))
        });
      }

        attrgen!(parsers);

        Err(original.error("unknown attribute"))
    }
}

attrgen!(gen_bindgen_attr);
