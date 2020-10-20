#![warn(missing_docs, clippy::all, clippy::pedantic, clippy::cargo)]
#![deny(broken_intra_doc_links, missing_debug_implementations)]

//! Derive macro for the `dispose` crate.
//!
//! This crate provides a derive macro for quickly deriving `Dispose` on types where the values can
//! be consumed relatively trivially.

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{emit_error, proc_macro_error};
use quote::quote_spanned;
use syn::{
    parse_macro_input, spanned::Spanned, Data, DataEnum, DataStruct, DeriveInput, Field, Fields,
    Ident, Index, Member,
};

mod field_attr;
mod with_val;
// mod item_attr;

use field_attr::*;
use with_val::*;
// use item_attr::*;

type Result<T, E = ()> = std::result::Result<T, E>;

/// Add trivial `Dispose` support to a struct or enum where the contained values implement
/// `Dispose` or `DisposeWith<W>`.
///
/// This macro is designed to reduce the boilerplate for writing custom containers housing
/// `Dispose` or `DisposeWith` resources.
///
/// # The `#[dispose]` attribute
///
/// The `#[dispose]` attribute available to types deriving `Dispose` provides four options for
/// decorating fields: `ignore`, `with`, `iter`, and `iter_with`.
///
/// - `#[dispose(ignore)]` is the simplest option.  It disables generating a `.dispose()` call for
///   the field it decorates.
/// - `#[dispose(with = <expr>)]` changes the `.dispose()` call to a `.dispose_with(...)` call that
///   is provided with a value determined by `<expr>`.  `expr` can take one of two forms: `.memb`
///   for a member access into `self`, or any other Rust expression, which will be token-pasted
///   into the `dispose_with` call as-is.
/// - `#[dispose(iter)]` changes the `.dispose()` call to `.dispose_iter()`, for types that
///   implement `DisposeIterator` rather than `Dispose`.
/// - `#[dispose(iter_with = <expr>)]` changes the `.dispose()` call to `.dispose_iter_with(...)`,
///   behaving similarly to both `#[dispose(iter)]` and `#[dispose(with = <expr>)]`.
///
/// # Examples
///
/// Here's a dead-simple example:
///
/// ```
/// use dispose::{prelude::*, Disposable};
///
/// struct MyResource {
///     important_stuff: String,
/// }
///
/// impl Dispose for MyResource {
///     fn dispose(self) {
///         println!("disposing {:?}", self.important_stuff);
///     }
/// }
///
/// struct MyOtherResource {
///     handle: u32,
/// }
///
/// impl Dispose for MyOtherResource {
///     fn dispose(self) {
///         println!("releasing handle {}", self.handle)
///     }
/// }
///
/// // The derive macro makes it trivial to implement Dispose on a container type for these
/// // resources.
/// #[derive(Dispose)]
/// struct MyContainer {
///     res: MyResource,
///     other: MyOtherResource,
/// }
///
/// impl MyContainer {
///     fn new(important_stuff: impl Into<String>, handle: u32) -> Disposable<Self> {
///         let important_stuff = important_stuff.into();
///
///         Self {
///             res: MyResource { important_stuff },
///             other: MyOtherResource { handle },
///         }.into()
///     }
/// }
///
/// {
///     let container = MyContainer::new("foobar", 27);
///
///     // Do some stuff with container here...
///     # let _ = container; // Silence any unused warnings.
/// }
/// // This prints:
/// // disposing "foobar"
/// // releasing handle 27
/// ```
///
/// Here's a more real-world example, using the gfx-hal crate:
///
/// ```no_run
/// # use dispose::{prelude::*, Disposable};
/// use gfx_hal::{prelude::*, Backend, device::Device};
///
/// // First, some setup - since this is non-trivial, the macro can't help here.
/// struct Buffer<B: Backend>(B::Buffer);
/// struct Memory<B: Backend>(B::Memory);
///
/// impl<B: Backend> DisposeWith<&B::Device> for Buffer<B> {
///     fn dispose_with(self, dev: &B::Device) { unsafe { dev.destroy_buffer(self.0) } }
/// }
///
/// impl<B: Backend> DisposeWith<&B::Device> for Memory<B> {
///     fn dispose_with(self, dev: &B::Device) { unsafe { dev.free_memory(self.0) } }
/// }
///
/// /// A single buffer with its own device memory allocation.
/// #[derive(Dispose)]
/// struct SingleBuffer<'a, B: Backend> {
///     #[dispose(ignore)]
///     dev: &'a B::Device,
///     #[dispose(with = .dev)]
///     buf: Buffer<B>,
///     #[dispose(with = .dev)]
///     mem: Memory<B>,
/// }
///
/// impl<'a, B: Backend> SingleBuffer<'a, B> {
///     fn new(dev: &'a B::Device, buf: Buffer<B>, mem: Memory<B>) -> Disposable<Self> {
///         Self { dev, buf, mem }.into()
///     }
/// }
///
/// /// A set of buffers sharing a single memory allocation.
/// #[derive(Dispose)]
/// struct MultiBuffer<'a, B: Backend> {
///     #[dispose(ignore)]
///     dev: &'a B::Device,
///     #[dispose(with = .dev)]
///     bufs: Vec<Buffer<B>>,
///     #[dispose(with = .dev)]
///     mem: Memory<B>,
/// }
///
/// impl<'a, B: Backend> MultiBuffer<'a, B> {
///     fn new(
///         dev: &'a B::Device,
///         bufs: impl IntoIterator<Item = Buffer<B>>,
///         mem: Memory<B>,
///     ) -> Disposable<Self> {
///         Self {
///             dev,
///             bufs: bufs.into_iter().collect(),
///             mem,
///         }.into()
///     }
/// }
/// #
/// # // Actually allocating these resources is beyond the scope of a documentation example, but I
/// # // did want to make this both realistic and doctest-able.
/// # fn create_buffer(_: &gfx_backend_empty::Device) -> Buffer<gfx_backend_empty::Backend> {
/// #     Buffer(())
/// # }
/// # fn alloc_memory(_: &gfx_backend_empty::Device) -> Memory<gfx_backend_empty::Backend> {
/// #     Memory(())
/// # }
/// #
/// # let a_device = &gfx_backend_empty::Device;
///
/// // Acquire a device here.
///
/// // Now we can create and manage a container for a single buffer...
/// let buf = SingleBuffer::new(
///     a_device,
///     create_buffer(a_device),
///     alloc_memory(a_device),
/// );
///
/// // ...or multiple buffers, with allocation sharing.
/// // And, there's no excessive copies of a_device stored in memory!
/// let bufs = MultiBuffer::new(
///     a_device,
///     (0..16).into_iter().map(|_| create_buffer(a_device)),
///     alloc_memory(a_device),
/// );
///
/// // Draw cool things with the buffers here...
/// # let _ = (buf, bufs); // Silence any unused warnings.
/// ```
#[proc_macro_error]
#[proc_macro_derive(Dispose, attributes(dispose))]
pub fn derive_dispose(item: TokenStream1) -> TokenStream1 {
    match derive_dispose_impl(parse_macro_input!(item)) {
        Ok(s) => s.into(),
        Err(()) => TokenStream1::new(),
    }
}

fn field_to_member(index: u32, field: &Field) -> Member {
    match &field.ident {
        Some(n) => Member::Named(n.clone()),
        None => Member::Unnamed(Index {
            index,
            span: field.span(),
        }),
    }
}

fn member_to_string(member: Member) -> String {
    match member {
        Member::Named(i) => i.to_string(),
        Member::Unnamed(i) => i.index.to_string(),
    }
}

fn derive_dispose_impl(input: DeriveInput) -> Result<TokenStream> {
    let span = input.span();
    let name = input.ident;

    for attr in input.attrs {
        if attr.path.is_ident("dispose") {
            emit_error! { span.unwrap(), "Unexpected #[dispose] attribute." };
        }
    }

    let generics = input.generics;
    let (impl_vars, ty_vars, where_clause) = generics.split_for_impl();

    let default_mode = FieldMode::Dispose { is_iter: false };

    let fn_body = match input.data {
        Data::Struct(s) => derive_dispose_struct(span, default_mode, s),
        Data::Enum(e) => derive_dispose_enum(span, default_mode, e),
        Data::Union(_) => {
            emit_error! { span.unwrap(), "Cannot derive Dispose on a union." }

            Err(())
        },
    }?;

    Ok(quote_spanned! { span =>
        impl #impl_vars ::dispose::Dispose for #name #ty_vars #where_clause {
            #[allow(non_snake_case, redundant_semicolons)]
            fn dispose(self) {
                #fn_body
            }
        }
    })
}

fn dispose_fields(
    span: Span,
    default_mode: FieldMode,
    fields: Fields,
    field_name: impl Fn(Span, Member) -> Ident + Copy,
) -> Result<TokenStream>
{
    let handle_field = |(id, field): (usize, Field)| {
        let span = field.span();
        let name = field_name(field.span(), field_to_member(id as u32, &field));

        let attr = parse_field_attrs(field.attrs).map_err(|_| ())?;
        let ty = field.ty;

        Ok(match attr.map_or(default_mode.clone(), |a| a.mode) {
            FieldMode::Dispose { is_iter } => {
                if is_iter {
                    quote_spanned! { span =>
                        <#ty as ::dispose::DisposeIterator>::dispose_iter(#name)
                    }
                } else {
                    quote_spanned! { span =>
                        <#ty as ::dispose::Dispose>::dispose(#name)
                    }
                }
            },
            FieldMode::DisposeWith { is_iter, with } => {
                let with = with.expand(field_name);

                if is_iter {
                    quote_spanned! { span =>
                        <#ty as ::dispose::DisposeIteratorWith<_>>
                            ::dispose_iter_with(#name, #with)
                    }
                } else {
                    quote_spanned! { span =>
                        <#ty as ::dispose::DisposeWith<_>>::dispose_with(#name, #with)
                    }
                }
            },
            FieldMode::Ignore => quote_spanned! { span => },
        })
    };

    let fields: Vec<_> = match fields {
        Fields::Named(n) => n
            .named
            .into_iter()
            .enumerate()
            .map(handle_field)
            .collect::<Result<Vec<_>>>()?,
        Fields::Unnamed(u) => u
            .unnamed
            .into_iter()
            .enumerate()
            .map(handle_field)
            .collect::<Result<Vec<_>>>()?,
        Fields::Unit => vec![],
    };

    Ok(quote_spanned! { span => #(#fields;)* })
}

fn destructure_fields(
    span: Span,
    fields: &Fields,
    field_name: impl Fn(Span, Member) -> Ident,
) -> TokenStream
{
    match fields {
        Fields::Named(n) => {
            let names = n.named.iter().enumerate().map(|(i, f)| {
                let var = field_name(f.span(), field_to_member(i as u32, &f));
                let ident = f.ident.clone();

                quote_spanned! { f.span() => #ident: #var }
            });

            quote_spanned! { span => { #(#names),* } }
        },
        Fields::Unnamed(u) => {
            let names = u
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, f)| field_name(f.span(), field_to_member(i as u32, f)));

            quote_spanned! { span => ( #(#names),* ) }
        },
        Fields::Unit => quote_spanned! { span => },
    }
}

fn derive_dispose_struct(
    span: Span,
    default_mode: FieldMode,
    data: DataStruct,
) -> Result<TokenStream>
{
    fn field_name(span: Span, member: Member) -> Ident {
        Ident::new(
            &format!("__dispose_self_f{}", member_to_string(member)),
            span,
        )
    }

    let names = destructure_fields(span, &data.fields, field_name);
    let fields = dispose_fields(span, default_mode, data.fields, field_name)?;

    Ok(quote_spanned! { span =>
        let Self #names = self;

        #fields
    })
}

fn derive_dispose_enum(span: Span, default_mode: FieldMode, data: DataEnum) -> Result<TokenStream> {
    fn field_name(span: Span, member: Member, var: impl AsRef<str>) -> Ident {
        Ident::new(
            &format!(
                "__dispose_self_v{}_f{}",
                var.as_ref(),
                member_to_string(member),
            ),
            span,
        )
    }

    let variants = data
        .variants
        .into_iter()
        .map(|var| {
            let name = var.ident;
            let name_str = name.to_string();

            let names = destructure_fields(span, &var.fields, |i, f| field_name(i, f, &name_str));
            let fields = dispose_fields(span, default_mode.clone(), var.fields, |i, f| {
                field_name(i, f, &name_str)
            })?;

            Ok(quote_spanned! { span =>
                Self::#name #names => {
                    #fields
                }
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote_spanned! { span =>
        match self {
            #(#variants),*
        }
    })
}
