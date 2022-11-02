use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use regex::Regex;
use syn::{parse_macro_input, DeriveInput, Type};

/// This function converts syn's string representation of typenames to
/// the format used by std::any::type_name.
fn canonicalized_type_name(ty: &Type) -> String {
    let type_name = quote! {#ty}.to_string();
    let re = Regex::new(r"\s*;\s*").expect("Bad regex");
    re.replace_all(&type_name, r"; ").to_string()
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

    inner_types.push(quote! {
        <#ty>::add_to_database(database)?;
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
                let type_name = canonicalized_type_name(&ty);
                fields.push(quote! {(#field_name, #type_name)});
            }
        }
        _ => panic!("Not a structure."),
    }

    let gen = quote! {
        impl bpf_script::types::AddToTypeDatabase for #name {
            fn add_to_database(database: &mut bpf_script::types::TypeDatabase) -> bpf_script::error::Result<usize> {
                const STRUCT_FIELDS: &[(&str, &str)] = &[#(#fields),*];
                #(#inner_types)*
                database.add_struct_by_names(Some(stringify!(#name)), STRUCT_FIELDS)
            }
        }
    };

    gen.into()
}
