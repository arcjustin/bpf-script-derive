//! [![Build Status](https://github.com/arcjustin/bpf-script-derive/workflows/build/badge.svg)](https://github.com/arcjustin/bpf-script-derive/actions?query=workflow%3Abuild)
//! [![crates.io](https://img.shields.io/crates/v/bpf-script-derive.svg)](https://crates.io/crates/bpf-script-derive)
//! [![mio](https://docs.rs/bpf-script-derive/badge.svg)](https://docs.rs/bpf-script-derive/)
//! [![Lines of Code](https://tokei.rs/b1/github/arcjustin/bpf-script-derive?category=code)](https://tokei.rs/b1/github/arcjustin/bpf-script-derive?category=code)
//!
//! Provides a derive macro for `AddToDatabase` to make adding Rust types to a `bpf_script::types::TypeDatabase` easier.
//!
//! ## License
//!
//! * [MIT license](http://opensource.org/licenses/MIT)
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, quote_spanned, ToTokens, TokenStreamExt};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Field, Ident, LitStr, Path, Type};

use darling::FromDeriveInput;

fn default_crate_root() -> Path {
    parse_quote!(::bpf_script)
}

#[derive(FromDeriveInput)]
#[darling(supports(struct_named))]
struct Receiver {
    ident: Ident,
    data: darling::ast::Data<(), Field>,
    /// The root to use for absolute imports.
    #[darling(rename = "crate", default = "default_crate_root")]
    crate_root: Path,
}

struct TypeRegistration<'a> {
    ty: &'a Type,
    database: &'a Ident,
}

impl ToTokens for TypeRegistration<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self { ty, database } = self;
        let name = local_ident(ty);
        // Set the span of the entire declaration to the type, so that a compile error
        // due to a missing trait impl will correctly point to the offending type.
        tokens.append_all(quote_spanned! {ty.span()=>
            let #name = <#ty>::add_to_database(#database)?;
        })
    }
}

/// Hashes a given value.
fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn local_ident(ty: &Type) -> Ident {
    let mut ident = format_ident!("local_{}", calculate_hash(ty));
    ident.set_span(ty.span());
    ident
}

/// Recursively adds inner types to the set of types for the database.
fn add_type<'a>(ty: &'a Type, set: &mut Vec<&'a Type>) {
    match ty {
        Type::Array(a) => add_type(&a.elem, set),
        Type::Tuple(t) => {
            for elem in &t.elems {
                add_type(elem, set);
            }
        }
        _ => (),
    };

    set.push(ty);
}

/// Implements AddToTypeDatabase for the type. If the type is a structure, it will
/// create a new type for the structure with the same name and all its inner fields.
#[proc_macro_derive(AddToTypeDatabase, attributes(field))]
pub fn derive_add_to_database(input: TokenStream) -> TokenStream {
    let receiver = match Receiver::from_derive_input(&parse_macro_input!(input)) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    // A list of all the types that are being registered with the database.
    // This list can deliberately include duplicates; these will end up
    // being shadowed variables. This duplication is needed because each
    // occurrence of a type will have a distinct span where the type was
    // encountered in the input, and deduping the types would only result
    // in the first occurrence of the type getting a compile error.
    let mut types = vec![];
    let mut fields = vec![];
    for Field { ident, ty, .. } in receiver
        .data
        .as_ref()
        .take_struct()
        .expect("Only named structs supported")
    {
        add_type(ty, &mut types);

        let field_name = LitStr::new(
            &ident.as_ref().expect("Field has no name").to_string(),
            ident.span(),
        );

        let local_name = local_ident(ty);

        fields.push(quote! {(#field_name, #local_name)});
    }

    let name = &receiver.ident;
    let crate_root = &receiver.crate_root;
    let db_param = Ident::new("database", Span::call_site());
    let registrations = types.iter().map(|ty| TypeRegistration {
        ty,
        database: &db_param,
    });

    let gen = quote! {
        #[automatically_derived]
        impl #crate_root::types::AddToTypeDatabase for #name {
            fn add_to_database(#db_param: &mut #crate_root::types::TypeDatabase) -> #crate_root::error::Result<usize> {
                #(#registrations)*
                #db_param.add_struct_by_ids(Some(stringify!(#name)), &[#(#fields),*])
            }
        }
    };

    gen.into()
}
