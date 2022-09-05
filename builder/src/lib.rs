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
    // 生成CommandBuilder struct中每个参数的set方法
    let set_data_token = generate_set_data_func_def(fields);
    // 在CommandBuilder中实现build方法返回Command对象
    let build_func_token = generate_build_original_object_def(name, fields);
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

        impl #builder_ident {
            #set_data_token
            #build_func_token
        }
    };
    expanded
}

fn get_builder_data_field(token: &DeriveInput) -> syn::Result<&Punctuated<Field, Token!(,)>> {
    match token.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref field_named) => {
                return Ok(&field_named.named)
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
    let types: Vec<_> = fields
        .iter()
        .map(|f| {
            if let Some(inner_ty) = get_option_inner_type(&f.ty) {
                quote! {
                    std::option::Option<#inner_ty>
                }
            } else {
                let origin_ty = &f.ty;
                quote! {
                    std::option::Option<#origin_ty>
                }
            }
        })
        .collect();

    let token_stream = quote! {
        #(#idents: #types),*
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

fn generate_set_data_func_def(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let idents: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let tys: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let mut final_token_stream = TokenStream::new();
    // let token_stream = quote! {
    //     #(
    //         pub fn #idents(&mut self, #idents: String) -> &mut Self {
    //             self.#idents = std::option::Option::Some(#idents);
    //             self
    //         }
    //     ),*
    // };
    // token_stream
    for (ident, type_) in idents.iter().zip(tys.iter()) {
        let tokenstream_piece;
        if let Some(inner_ty) = get_option_inner_type(type_) {
            tokenstream_piece = quote! {
                fn #ident(&mut self, #ident: #inner_ty) -> &mut Self {
                    self.#ident = std::option::Option::Some(#ident);
                    self
                }
            };
        } else {
            tokenstream_piece = quote! {
                fn #ident(&mut self, #ident: #type_) -> &mut Self {
                    self.#ident = std::option::Option::Some(#ident);
                    self
                }
            };
        }

        // 不断追加新的TokenStream片段到一个公共的TokenStream上
        final_token_stream.extend(tokenstream_piece);
    }
    final_token_stream
}

// 生成build方法的token对象
fn generate_build_original_object_def(
    name: &Ident,
    fields: &Punctuated<Field, Comma>,
) -> TokenStream {
    let idents: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let tys: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let mut check_token_vec = Vec::new();
    let mut result_token_vec = Vec::new();

    for (ident, type_) in idents.iter().zip(tys.iter()) {
        if get_option_inner_type(type_).is_none() {
            let check_token = quote! {
                if self.#ident.is_none() {

                        let err = format!("{} field missing",  stringify!(#ident));
                        return std::result::Result::Err(err.into());
                    }
            };
            check_token_vec.push(check_token);
        }
    }
    for (ident, type_) in idents.iter().zip(tys.iter()) {
        if get_option_inner_type(type_).is_none() {
            result_token_vec.push(quote! {
                #ident: self.#ident.clone().unwrap()
            });
        } else {
            result_token_vec.push(quote! {
                #ident: self.#ident.clone()
            });
        }
    }
    let token = quote! {
        pub fn build(&mut self) -> Result<#name, std::boxed::Box<dyn std::error::Error>> {
            #(#check_token_vec)*
            let ret = #name {
                #(#result_token_vec),*
            };
            std::result::Result::Ok(ret)
        }
    };
    token
}

fn get_option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = ty
    {
        if let Some(seg) = segments.last() {
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    ref args,
                    ..
                }) = seg.arguments
                {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}
