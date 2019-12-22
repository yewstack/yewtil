use proc_macro2::{TokenStream, Ident, Span};
use proc_macro::TokenStream as TokenStream1;
use syn::{Visibility, Error, Field, Stmt, Block, VisPublic};
use syn::Token;
use syn::token;
use syn::punctuated::Punctuated;
use syn::parse::{Parse, ParseBuffer};
use syn::{parenthesized, braced};
use syn::parse_macro_input;
use syn::export::ToTokens;
use quote::quote;

pub fn function_component_handler(attr: TokenStream, item: TokenStream1) -> TokenStream1 {
    let component_name = attr.to_string();
    assert!(!component_name.is_empty(), "you must provide a component name. eg: function_component(MyComponent)");
    let component_name = Ident::new(&component_name, Span::call_site());

    let item_copy = item.clone();

    let function = parse_macro_input!(item_copy as Function);

    TokenStream1::from(FunctionComponentInfo {
        component_name,
        function
    }.to_token_stream())
}

pub struct FunctionComponentInfo {
    component_name: Ident,
    function: Function
}


// TODO, support type parameters

pub struct Function {
    pub vis: Visibility,
    pub fn_token: Token![fn],
    pub name: Ident,
    pub paren_token: token::Paren,
    pub fields: Punctuated<Field, Token![,]>,
    pub returns_token: Token![->],
    pub return_ty: Ident,
    pub brace_token: token::Brace,
    pub body: Vec<Stmt>
}

impl Parse for Function {
    fn parse(input: &ParseBuffer) -> Result<Self, Error> {
        let content;
        let content2;
        Ok(Function {
            vis: input.parse()?,
            fn_token: input.parse()?,
            name: input.parse()?,
            paren_token: parenthesized!(content in input),
            fields: content.parse_terminated(Field::parse_named)?,
            returns_token: input.parse()?,
            return_ty: input.parse()?,
            brace_token: braced!(content2 in input),
            body: content2.call(Block::parse_within)?
        })
    }
}

impl ToTokens for Function {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Function {
            vis, fn_token, name, fields, returns_token, return_ty,  body, ..
        } = self;
        let fields = fields.iter()
            .map(|field: &Field| {
                let mut new_field: Field = field.clone();
                new_field.attrs = vec![];
                new_field
            })
            .collect::<Punctuated<_, Token![,]>>();

        tokens.extend(quote! {
            #vis #fn_token #name(#fields) #returns_token #return_ty {
                #(#body)*
            }
        })
    }
}

impl ToTokens for FunctionComponentInfo {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FunctionComponentInfo {
            component_name, function
        } = self;
        let function_impl = quote!{#function};
        let Function {
            name, fields, ..
        } = function;

        let impl_name = format!("Pure{}", component_name.to_string());
        let impl_name = Ident::new(&impl_name, Span::call_site());

        let alias = quote! {
            pub type #component_name = Pure<#impl_name>;
        };

        // Set the fields to be public
        let public_fields = fields.iter()
            .map(|field: &Field| {
                let mut new_field: Field = field.clone();
                let visibility = Visibility::Public(VisPublic{ pub_token: syn::token::Pub {span: Span::call_site()} });
                new_field.vis = visibility;
                new_field
            })
            .collect::<Punctuated<_, Token![,]>>();


        let component_struct = quote!{
            #[derive(Clone, PartialEq, Properties)]
            pub struct #impl_name {
                #public_fields
            }
        };

        let arguments = fields.iter()
            .map(|field| {
                let field = field.ident.as_ref().expect("Field must have name");
                quote! {
                    self.#field.clone() // TODO this clone here is expensive, instead have the function take &refs and strip them when making the component struct impl.
                }
            })
            .collect::<Punctuated<_, Token![,]>>();

        let pure_component_impl = quote! {
            impl PureComponent for #impl_name {
                fn render(&self) -> VNode {
                     #name(#arguments)
                }
            }
        };


        tokens.extend(quote!{
            #function_impl
            #alias
            #component_struct
            #pure_component_impl
        })
    }
}