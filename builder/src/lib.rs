use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, token::Comma, Data, DeriveInput,
    Field, Fields, Ident, Token,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_token = parse_macro_input!(input as DeriveInput);

    let expanded = get_builder_expanded(&input_token);

    proc_macro::TokenStream::from(expanded)
}

// Hand the output tokens back to the compiler.
fn get_builder_expanded(token: &DeriveInput) -> TokenStream {
    let name = &token.ident;
    let builder_name = format!("{}Builder", name);
    let span = token.span();
    let builder_ident = Ident::new(&builder_name, span);
    // get the fields info like below
    // TokenStream [
    // Ident {
    //     ident: "executable",
    //     span: #0 bytes(220..230),
    // },
    // Punct {
    //     ch: ':',
    //     spacing: Alone,
    //     span: #0 bytes(230..231),
    // },
    // Ident {
    //     ident: "String",
    //     span: #0 bytes(232..238),
    // },
    // ...nip...
    let fields = get_builder_data_field(token).unwrap();
    // get field define
    let field_token = generate_builder_struct_fields_def(fields);
    // init builder values
    let init_field_token = generate_builder_struct_factory_init_clauses(fields);
    let expanded = quote! {
        // The generated impl.
        impl #name {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #init_field_token
                }
            }
        }

        #[derive(Debug)]
        pub struct #builder_ident {
            #field_token
        }
    };
    expanded
}

fn get_builder_data_field(token: &DeriveInput) -> syn::Result<&Punctuated<Field, Token!(,)>> {
    match token.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fieldNamed) => {
                return Ok(&fieldNamed.named)
            }
            _ => Err(syn::Error::new_spanned(
                token,
                "Must define Named fields, not Unnamed fields of a tuple struct or tuple variant or Unit struct or unit variant ".to_string(),
            )),
        },
        _ => Err(syn::Error::new_spanned(
            token,
            "Must define on a Struct, not Enum or Union".to_string(),
        )),
    }
}

fn generate_builder_struct_fields_def(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let idents: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let token_stream = quote! {
        #(#idents: std::option::Option<#types>),*
    };
    token_stream
}

fn generate_builder_struct_factory_init_clauses(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let idents: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let token_stream = quote! {
        #(#idents: std::option::Option::None),*
    };
    token_stream
}
