extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(VariantsIter)]
pub fn derive_variants_iter(item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as DeriveInput);
    let variants = if let syn::Data::Enum(vars) = &item.data {
        vars.variants
            .iter()
            .map(|v| v.ident.clone())
            .collect::<Vec<_>>()
    } else {
        panic!("VariantsIter can only be applied to enum");
    };

    let enum_name = item.ident;

    quote! {
        impl #enum_name {
            pub const VARIANTS: &'static [Self] = &[ #(Self::#variants),* ];

            pub fn iter() -> impl Iterator<Item = &'static Self> {
                Self::VARIANTS.into_iter()
            }
        }
    }
    .into()
}
