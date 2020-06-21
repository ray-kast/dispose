use super::WithVal;
use syn::{
    ext::IdentExt,
    parenthesized,
    parse::{Error as ParseError, Parse, ParseStream, Parser, Result as ParseResult},
    spanned::Spanned,
    token, AttrStyle, Attribute, Ident, Token,
};

#[derive(Debug, Clone)]
pub struct FieldAttr {
    pub mode: FieldMode,
}

#[derive(Debug, Clone)]
pub enum FieldMode {
    Dispose { is_iter: bool },
    DisposeWith { is_iter: bool, with: WithVal },
    Ignore,
}

impl Default for FieldAttr {
    fn default() -> Self {
        Self {
            mode: FieldMode::default(),
        }
    }
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
                        ))
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
) -> ParseResult<Option<FieldAttr>> {
    let mut ret = Ok(None);
    let mut n = 0;

    for attr in attrs {
        let span = attr.span();

        if attr.style != AttrStyle::Outer {
            span.unwrap().error("Unexpected inner attribute").emit();
        }

        if attr.path.is_ident("dispose") {
            if n > 0 {
                span.unwrap().error("Duplicate #[dispose] attribute").emit();

                ret = Err(ParseError::new(span, "Duplicate #[dispose] attribute"));
            } else {
                ret = match Parser::parse2(FieldAttr::parse, attr.tokens) {
                    Ok(a) => Ok(Some(a)),
                    Err(e) => {
                        e.span()
                            .unwrap()
                            .error(format!("Failed to parse #[dispose] attribute: {}", e))
                            .emit();

                        Err(e)
                    },
                }
            }

            n += 1;
        }
    }

    ret
}
