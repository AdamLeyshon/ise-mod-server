use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_derive(FieldCount)]
pub fn derive_field_count(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    let field_count = input.fields.iter().count();

    let name = &input.ident;

    let output = quote! {
        impl #name {
            pub fn field_count() -> usize {
                #field_count
            }

            pub fn batch_size() -> usize {
                (65_535usize / #field_count) - 1
            }
        }
    };

    // Return output tokenstream
    TokenStream::from(output)
}
