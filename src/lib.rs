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

/// Recursively adds inner types to the auto_types list.
fn add_type(ty: &Type, auto_types: &mut Vec<TokenStream2>) {
    match ty {
        Type::Array(a) => add_type(&a.elem, auto_types),
        Type::Tuple(t) => {
            for elem in &t.elems {
                add_type(elem, auto_types);
            }
        }
        _ => (),
    }

    auto_types.push(quote! {
        <#ty>::add_to_btf(btf)?;
    });
}

/// Implements AddToBtf for the type. If the type is a structure, it will
/// create a new BTF type for the structure with the same name and all its
/// inner field types.
#[proc_macro_derive(AddToBtf, attributes(field))]
pub fn derive_add_to_bpf(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = &input.ident;

    let mut auto_types = vec![];
    let mut fields = vec![];
    match input.data {
        syn::Data::Struct(s) => {
            for field in s.fields {
                let field_name = field.ident.expect("Field has no name").to_string();
                let ty = field.ty;
                add_type(&ty, &mut auto_types);
                let type_name = canonicalized_type_name(&ty);
                fields.push(quote! {(#field_name, #type_name)})
            }
        }
        _ => panic!("Not a structure."),
    }

    let gen = quote! {
        impl AddToBtf for #name {
            fn add_to_btf(btf: &mut btf::BtfTypes) -> Option<&btf::types::Type> {
                const STRUCT_FIELDS: &[(&str, &str)] = &[#(#fields),*];
                usize::add_to_btf(btf)?;
                #(#auto_types)*
                btf.add_struct(stringify!(#name), STRUCT_FIELDS)
            }
        }
    };

    gen.into()
}
