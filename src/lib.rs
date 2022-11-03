//! [![Build Status](https://github.com/arcjustin/bpf-script-derive/workflows/build/badge.svg)](https://github.com/arcjustin/bpf-script-derive/actions?query=workflow%3Abuild)
//! [![crates.io](https://img.shields.io/crates/v/bpf-script-derive.svg)](https://crates.io/crates/bpf-script-derive)
//! [![mio](https://docs.rs/bpf-script-derive/badge.svg)](https://docs.rs/bpf-script-derive/)
//! [![Lines of Code](https://tokei.rs/b1/github/arcjustin/bpf-script-derive?category=code)](https://tokei.rs/b1/github/arcjustin/bpf-script-derive?category=code)
//!
//! Provides a derive macro for `AddToDatabase` to make adding Rust types to a `bpf_script::types::TypeDatabase` easier.
//!
//! ## Usage
//!
//! For usage examples, see code located in [examples/](examples/) :
//!
//!   | Examples | Description |
//!   |----------|-------------|
//!   |[custom-type](examples/custom-type.rs)| Creates and inserts a custom type into an empty BTF database|
//!
//! ## License
//!
//! * [MIT license](http://opensource.org/licenses/MIT)
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use syn::{parse_macro_input, DeriveInput, Type};

/// Hashes a given value.
fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// Recursively adds inner types to the inner_types list.
fn add_type(ty: &Type, inner_types: &mut Vec<TokenStream2>) {
    match ty {
        Type::Array(a) => add_type(&a.elem, inner_types),
        Type::Tuple(t) => {
            for elem in &t.elems {
                add_type(elem, inner_types);
            }
        }
        _ => (),
    }

    let local_name = format_ident!("local_{}", calculate_hash(&ty));
    inner_types.push(quote! {
        let #local_name = <#ty>::add_to_database(database)?;
    });
}

/// Implements AddToTypeDatabase for the type. If the type is a structure, it will
/// create a new type for the structure with the same name and all its inner fields.
#[proc_macro_derive(AddToTypeDatabase, attributes(field))]
pub fn derive_add_to_database(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = &input.ident;

    let mut inner_types = vec![];
    let mut fields = vec![];
    match input.data {
        syn::Data::Struct(s) => {
            for field in s.fields {
                let field_name = field.ident.expect("Field has no name").to_string();
                let ty = field.ty;
                add_type(&ty, &mut inner_types);
                let local_name = format_ident!("local_{}", calculate_hash(&ty));
                fields.push(quote! {(#field_name, #local_name)});
            }
        }
        _ => panic!("Not a structure."),
    }

    let gen = quote! {
        impl bpf_script::types::AddToTypeDatabase for #name {
            fn add_to_database(database: &mut bpf_script::types::TypeDatabase) -> bpf_script::error::Result<usize> {
                #(#inner_types)*
                let struct_fields = [#(#fields),*].to_vec();
                database.add_struct_by_ids(Some(stringify!(#name)), struct_fields.as_slice())
            }
        }
    };

    gen.into()
}
