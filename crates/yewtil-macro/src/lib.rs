extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use quote::ToTokens;
use std::convert::TryFrom;
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Error, Field, Type};

mod util;
use util::extract_type_from_callback;
use std::fmt;
use crate::function_component::function_component_handler;

mod function_component;
#[proc_macro_attribute]
pub fn function_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    function_component_handler(attr.into(), item.into()).into()
}

#[proc_macro_derive(Emissive, attributes(props))]
pub fn emissive(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveEmissiveInput);
    TokenStream::from(input.into_token_stream())
}

// TODO add full debug support back on this type, Apparently `Type` doesn't implement it,
// but it should...
struct CallbackField {
    name: Ident,
    message_ty: Type,
}

impl fmt::Debug for CallbackField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CallbackField")
            .field("name", &self.name)
//            .field("message_ty", &self.message_ty.)
            .finish()
    }
}

impl TryFrom<Field> for CallbackField {
    type Error = Error;

    fn try_from(field: Field) -> Result<Self> {
        let message_ty = extract_type_from_callback(&field.ty)
            .map(Clone::clone)
            .ok_or_else(|| {
                syn::Error::new(
                    field.span(),
                    "Annotated \"emissive\" field was not a ::yew::Callback.",
                )
            })?;

        Ok(CallbackField {
            name: field.ident.unwrap(),
            message_ty,
        })
    }
}

#[derive(Debug)]
struct DeriveEmissiveInput {
    struct_name: Ident,
    callback_field: Option<CallbackField>,
}

impl Parse for DeriveEmissiveInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let input: DeriveInput = input.parse()?;
        let named_fields = match input.data {
            syn::Data::Struct(data) => match data.fields {
                syn::Fields::Named(fields) => fields.named,
                _ => unimplemented!("only structs are supported"),
            },
            _ => unimplemented!("only structs are supported"),
        };

        let named_fields_span = named_fields.span();

        let callback_fields: Vec<CallbackField> = named_fields
            .into_iter()
            .filter_map(|x| CallbackField::try_from(x).ok())
            .collect::<Vec<CallbackField>>();

        if callback_fields.len() > 1 {
            return Err(syn::Error::new(named_fields_span, "There can only be one `Callback<_>` field. If you want to support more than one, you must manually implement `Emissive`."));
        }
        let callback_field = callback_fields.into_iter().next();

        Ok(DeriveEmissiveInput {
            struct_name: input.ident,
            callback_field,
        })
    }
}

impl ToTokens for DeriveEmissiveInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            struct_name,
            callback_field,
        } = self;

        let message_ty = if let Some(cb_field) = callback_field {
            let message_ty = &cb_field.message_ty;
            quote! {
                type Message = #message_ty;
            }
        } else {
            quote! {
                type Message = ();
            }
        };

        let emit_fn = if let Some(cb_field) = callback_field {
            let name = &cb_field.name;
            quote! {
                fn emit(&self, msg: Self::Message) {
                    self.#name.emit(msg)
                }
            }
        } else {
            quote! {
                fn emit(&self, msg: Self::Message) {}
            }
        };

        let emissive_impl = quote! {
            impl ::yewtil::Emissive for #struct_name {
                #message_ty
                #emit_fn
            }
        };

        tokens.extend(emissive_impl);
    }
}
