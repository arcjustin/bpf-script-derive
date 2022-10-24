use darling::FromField;
use proc_macro::{self, TokenStream};
use quote::{format_ident, quote};
use regex::Regex;
use syn::{parse_macro_input, DeriveInput};

#[derive(FromField, Default)]
#[darling(default, attributes(field))]
struct Opts {
    name: Option<String>,
    type_name: Option<String>,
}

#[proc_macro_derive(AddToBtf, attributes(field))]
pub fn derive_add_to_bpf(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = &input.ident;

    let type_name_re = Regex::new(r"^([a-zA-Z0-9_]+)(\[(\d+)\])?").expect("Bad type_name regex");
    let mut arrays = vec![];
    let mut auto_types = vec![];
    let mut fields = vec![];
    match input.data {
        syn::Data::Struct(s) => {
            for field in s.fields {
                let opts = Opts::from_field(&field).expect("Wrong options");

                let field_name = match opts.name {
                    Some(name) => name,
                    None => field.ident.expect("Field has no name").to_string(),
                };

                if let Some(full_type_name) = opts.type_name {
                    let captures = type_name_re
                        .captures(&full_type_name)
                        .expect("Matching on field failed");
                    let type_name = captures.get(1).expect("Bad type_name format.").as_str();
                    let field_type_name = if let Some(num_elements) = captures.get(3) {
                        let array_name = format_ident!("{}_{}_{}", name, field_name, type_name);
                        let num_elements = num_elements
                            .as_str()
                            .parse::<u32>()
                            .expect("Array size invalid");
                        arrays.push(quote! {(stringify!(#array_name), #type_name, #num_elements)});
                        array_name
                    } else {
                        format_ident!("{}", type_name)
                    };

                    fields.push(quote! {(#field_name, stringify!(#field_type_name))});
                } else {
                    let ty = field.ty;
                    auto_types.push(quote! {
                        <#ty>::add_to_btf(btf)?;
                    });
                    fields.push(quote! {(#field_name, stringify!(#ty))})
                }
            }
        }
        _ => panic!("Not a structure."),
    }

    let gen = quote! {
        impl AddToBtf for #name {
            fn add_to_btf(btf: &mut btf::BtfTypes) -> Option<&btf::types::Type> {
                const ARRAY_TYPES: &[(&str, &str, u32)] = &[#(#arrays),*];
                const STRUCT_FIELDS: &[(&str, &str)] = &[#(#fields),*];
                usize::add_to_btf(btf)?;
                for array_type in ARRAY_TYPES {
                    btf.add_array(array_type.0, "usize", array_type.1, array_type.2);
                }
                #(#auto_types)*
                btf.add_struct(stringify!(#name), STRUCT_FIELDS)
            }
        }
    };

    gen.into()
}
