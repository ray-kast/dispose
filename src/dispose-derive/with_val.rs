use proc_macro2::Span;
use syn::{
    fold::Fold,
    parse::{Parse, ParseStream, Result as ParseResult},
    parse_quote,
    spanned::Spanned,
    Expr, ExprPath, Ident, Item, Member, Token,
};

#[derive(Debug, Clone)]
pub enum WithVal {
    Expr(Expr),
    SelfDot(Token![.], Expr),
}

struct ExpandSelf<F: Fn(Span, Member) -> Ident>(F);

impl<F: Fn(Span, Member) -> Ident> Fold for ExpandSelf<F> {
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        match expr {
            Expr::Field(f)
                if match &*f.base {
                    Expr::Path(p) if p.path.is_ident("self") => true,
                    _ => false,
                } =>
            {
                let span = f.span();

                Expr::Path(ExprPath {
                    attrs: f.attrs,
                    qself: None,
                    path: (self.0)(span, f.member).into(),
                })
            }
            e => syn::fold::fold_expr(self, e),
        }
    }

    // For hygiene reasons.  Any item will clear the meaning of self, so don't descend into them.
    fn fold_item(&mut self, item: Item) -> Item { item }
}

impl WithVal {
    // TODO: this may produce confusing errors if a requested member doesn't exist
    pub fn expand(self, field_name: impl Fn(Span, Member) -> Ident) -> Expr {
        ExpandSelf(field_name).fold_expr(match self {
            Self::Expr(e) => e,
            Self::SelfDot(d, e) => parse_quote! { self #d #e },
        })
    }
}

impl Parse for WithVal {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        if input.peek(Token![.]) {
            Ok(Self::SelfDot(input.parse()?, input.parse()?))
        } else {
            Ok(Self::Expr(input.parse()?))
        }
    }
}
