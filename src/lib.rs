use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use syn::{parse_macro_input, DeriveInput};

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
                let type_name = quote! {#ty}.to_string();
                let re = Regex::new(r"^\[\s*([^ ;]+)\s*;\s*(\d+)\s*\]$").expect("Bad regex");
                let type_name = re.replace(&type_name, r"[$1; $2]");
                auto_types.push(quote! {
                    <#ty>::add_to_btf(btf)?;
                });
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
