extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, Field, Fields, Ident, Token, Variant, parse_macro_input, punctuated::Punctuated,
    token::Comma,
};

#[proc_macro_derive(ArrowMessage)]
pub fn from_into_arrow_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    match input.data {
        syn::Data::Struct(s) => match s.fields {
            Fields::Named(ref fields) => struct_derive(name, fields.named.clone()),
            _ => panic!("Only structs with named fields are supported"),
        },
        syn::Data::Enum(e) => enum_derive(name, e.variants.clone()),
        _ => panic!("Only structs and enums are supported"),
    }
}

fn struct_derive(name: Ident, fields: Punctuated<Field, Comma>) -> TokenStream {
    let field_attributes = fields
        .iter()
        .map(|field| (&field.ident, &field.ty))
        .collect::<Vec<_>>();

    let fields_fill = field_attributes.iter().map(|&(field, ty)| {
        quote! {
            <#ty>::field(stringify!(#field)),
        }
    });

    let union_data_fill = field_attributes.iter().map(|&(field, _)| {
        quote! {
            #field: arrow_message::prelude::extract_union_data(stringify!(#field), &map, &children)?,
        }
    });

    let arrow_data_fill = field_attributes.iter().map(|&(field, _)| {
        quote! {
            self.#field.try_into_arrow()?,
        }
    });

    let expanded = quote! {
        impl arrow_message::prelude::ArrowMessage for #name {
            fn field(name: impl Into<String>) -> Field {
                arrow_message::prelude::make_union_fields(
                    name,
                    vec![
                        #(#fields_fill)*
                    ],
                )
            }

            fn try_from_arrow(data: arrow::array::ArrayData) -> Result<Self, arrow::error::ArrowError>
            where
                Self: Sized,
            {
                let (map, children) = arrow_message::prelude::unpack_union(data);

                Ok(Self {
                    #(#union_data_fill)*
                })
            }

            fn try_into_arrow(self) -> Result<arrow::array::ArrayRef, arrow::error::ArrowError> {
                let union_fields = arrow_message::prelude::get_union_fields::<Self>()?;

                make_union_array(
                    union_fields,
                    vec![
                        #(#arrow_data_fill)*
                    ],
                )
            }
        }

        impl TryFrom<arrow::array::ArrayData> for #name {
            type Error = arrow::error::ArrowError;

            fn try_from(data: arrow::array::ArrayData) -> Result<Self, Self::Error> {
                #name::try_from_arrow(data)
            }
        }

        impl TryFrom<#name> for arrow::array::ArrayData {
            type Error = arrow::error::ArrowError;

            fn try_from(item: #name) -> Result<Self, Self::Error> {
                item.try_into_arrow().map(|array| array.into_data())
            }
        }
    };

    TokenStream::from(expanded)
}

fn enum_derive(name: Ident, variants: Punctuated<Variant, Token![,]>) -> TokenStream {
    let variants: Vec<_> = variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let variant_str = variant_name.to_string().to_lowercase(); // Exemple : `Foo` -> "foo"
            (variant_name, variant_str)
        })
        .collect();

    let into_string_arms = variants.iter().map(|(variant_name, variant_str)| {
        quote! {
            #name::#variant_name => #variant_str.to_string(),
        }
    });

    let try_from_string_arms = variants.iter().map(|(variant_name, variant_str)| {
        quote! {
            #variant_str => Ok(#name::#variant_name),
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn into_string(self) -> String {
                match self {
                    #(#into_string_arms)*
                }
            }

            pub fn try_from_string(s: String) -> miette::Result<Self, ArrowError> {
                match s.as_str() {
                    #(#try_from_string_arms)*
                    _ => Err(ArrowError::ParseError(format!("Invalid value for {}: {}", stringify!(#name), s))),
                }
            }
        }

        impl arrow_message::prelude::ArrowMessage for #name {
            fn field(name: impl Into<String>) -> Field {
                String::field(name)
            }

            fn try_from_arrow(data: arrow::array::ArrayData) -> miette::Result<Self, arrow::error::ArrowError>
            where
                Self: Sized,
            {
                Encoding::try_from_string(String::try_from_arrow(data)?)
            }

            fn try_into_arrow(self) -> miette::Result<arrow::array::ArrayRef, arrow::error::ArrowError> {
                String::try_into_arrow(self.into_string())
            }
        }


        impl TryFrom<arrow::array::ArrayData> for #name {
            type Error = arrow::error::ArrowError;

            fn try_from(data: arrow::array::ArrayData) -> Result<Self, Self::Error> {
                #name::try_from_arrow(data)
            }
        }

        impl TryFrom<#name> for arrow::array::ArrayData {
            type Error = arrow::error::ArrowError;

            fn try_from(item: #name) -> Result<Self, Self::Error> {
                item.try_into_arrow().map(|array| array.into_data())
            }
        }
    };

    TokenStream::from(expanded)
}
