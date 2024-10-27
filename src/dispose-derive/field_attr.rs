use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    ext::IdentExt,
    parenthesized,
    parse::{Error as ParseError, Parse, ParseStream, Parser, Result as ParseResult},
    spanned::Spanned,
    token, AttrStyle, Attribute, Ident, Token,
};

use super::WithVal;

#[derive(Debug, Clone, Default)]
pub struct FieldAttr {
    pub mode: FieldMode,
}

#[derive(Debug, Clone)]
pub enum FieldMode {
    Dispose { is_iter: bool },
    DisposeWith { is_iter: bool, with: WithVal },
    Ignore,
}

impl Default for FieldMode {
    fn default() -> Self { FieldMode::Dispose { is_iter: false } }
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        if input.peek(token::Paren) {
            let arg;
            parenthesized!(arg in input);

            if arg.peek(Ident::peek_any) {
                let ident = arg.call(Ident::parse_any)?;

                let mode = match ident {
                    i if i == "ignore" => FieldMode::Ignore,
                    i if i == "with" => {
                        arg.parse::<Token![=]>()?;

                        FieldMode::DisposeWith {
                            is_iter: false,
                            with: arg.parse()?,
                        }
                    },
                    i if i == "iter" => FieldMode::Dispose { is_iter: true },
                    i if i == "iter_with" => {
                        arg.parse::<Token![=]>()?;

                        FieldMode::DisposeWith {
                            is_iter: true,
                            with: arg.parse()?,
                        }
                    },
                    i => {
                        return Err(ParseError::new(
                            i.span(),
                            "expected `ignore`, `with`, `iter`, or `iter_with`",
                        ));
                    },
                };

                Ok(Self { mode })
            } else {
                Ok(Self::default())
            }
        } else {
            Ok(Self::default())
        }
    }
}

pub fn parse_field_attrs<I: IntoIterator<Item = Attribute>>(
    attrs: I,
    diag: &mut TokenStream,
) -> ParseResult<Option<FieldAttr>> {
    let mut ret = Ok(None);
    let mut n = 0;

    for attr in attrs {
        let span = attr.span();

        if attr.style != AttrStyle::Outer {
            diag.extend(
                syn::Error::new(span.unwrap().into(), "Unexpected inner attribute")
                    .to_compile_error(),
            );
        }

        if attr.path().is_ident("dispose") {
            if n > 0 {
                diag.extend(
                    syn::Error::new(span.unwrap().into(), "Duplicate #[dispose] attribute")
                        .to_compile_error(),
                );

                ret = Err(ParseError::new(span, "Duplicate #[dispose] attribute"));
            } else {
                // TODO: using ToTokens is stupid and you know it
                ret = match Parser::parse2(FieldAttr::parse, attr.meta.to_token_stream()) {
                    Ok(a) => Ok(Some(a)),
                    Err(e) => {
                        diag.extend(
                            syn::Error::new(
                                span.unwrap().into(),
                                format!("Failed to parse #[dispose] attribute: {e}"),
                            )
                            .to_compile_error(),
                        );

                        Err(e)
                    },
                }
            }

            n += 1;
        }
    }

    ret
}
