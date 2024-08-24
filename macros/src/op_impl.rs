use darling::{FromDeriveInput, FromField, FromMeta};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{Path, Token, Type};

#[derive(Debug)]
pub struct OpAttrs {
    pub attrs: Vec<Attr>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Attr(pub syn::Ident, pub syn::Type);

fn path_is_option(path: &Path) -> bool {
    path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments.iter().next().unwrap().ident == "Option"
}

fn type_is_option(ty: &syn::Type) -> bool {
    match ty {
        Type::Path(ty_path) => path_is_option(&ty_path.path),
        _ => false,
    }
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attr_name = input.parse::<syn::Ident>()?;
        input.parse::<Token![:]>()?;
        let attr_ty = input.parse::<syn::Type>()?;

        Ok(Self(attr_name, attr_ty))
    }
}

impl FromMeta for OpAttrs {
    fn from_meta(item: &syn::Meta) -> darling::Result<Self> {
        if let syn::Meta::List(list) = item {
            let parser = Punctuated::<Attr, Token![,]>::parse_separated_nonempty;
            let tokens = list.tokens.clone();
            let attrs = parser
                .parse(tokens.into())?
                .iter()
                .cloned()
                .collect::<Vec<Attr>>();

            return Ok(OpAttrs { attrs });
        }
        // I genuinely have no idea what kind of error to put here
        panic!("expected syn::MetaList");
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(operation), supports(struct_named))]
pub struct OpReceiver {
    pub ident: syn::Ident,
    pub data: darling::ast::Data<(), OpFieldReceiver>,
    pub name: String,
    pub dialect: syn::Ident,
    #[darling(default)]
    pub known_attrs: Option<OpAttrs>,
}

#[derive(Default, Debug, FromMeta)]
pub struct RegionAttrs {
    #[darling(default)]
    pub single_block: bool,
    #[darling(default)]
    pub no_args: bool,
}

fn parse_region_attrs(attr: &syn::Attribute) -> Option<RegionAttrs> {
    if !attr.path().is_ident("region") {
        return None;
    }

    if let syn::Meta::Path(_) = &attr.meta {
        Some(RegionAttrs::default())
    } else {
        RegionAttrs::from_meta(&attr.meta).ok()
    }
}

#[derive(Debug)]
pub enum OpFieldAttrs {
    Region(RegionAttrs),
    Operand,
    Return,
    None,
}

fn transform_field_attrs(attrs: Vec<syn::Attribute>) -> darling::Result<OpFieldAttrs> {
    for attr in attrs {
        if let Some(region) = parse_region_attrs(&attr) {
            return Ok(OpFieldAttrs::Region(region));
        }
        if attr.path().is_ident("ret_type") {
            return Ok(OpFieldAttrs::Return);
        }
        if attr.path().is_ident("operand") {
            return Ok(OpFieldAttrs::Operand);
        }
    }

    Ok(OpFieldAttrs::None)
}

#[derive(Debug, FromField)]
#[darling(forward_attrs(region, ret_type, operand))]
pub struct OpFieldReceiver {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
    #[darling(with = transform_field_attrs)]
    pub attrs: OpFieldAttrs,
}

pub fn build_attr_accessors(attrs: &[Attr]) -> proc_macro2::TokenStream {
    let mut attr_accessors = vec![];

    for attr in attrs {
        let getter_name = format_ident!("get_{}_attr", attr.0);
        let setter_name = format_ident!("set_{}_attr", attr.0);
        let attr_str = attr.0.to_string();

        if type_is_option(&attr.1) {
            attr_accessors.push(quote! {
                pub fn #getter_name(&self) -> Option<tir_core::Attr> {
                    self.r#impl.attrs.get(#attr_str).cloned()
                }

                pub fn #setter_name<T>(&mut self, value: Option<T>) where tir_core::Attr: From<T> {
                    match value {
                        Some(value) => {
                            let attr = tir_core::Attr::from(value);
                            self.r#impl.attrs.insert(#attr_str.to_string(), attr);
                        },
                        None => { self.r#impl.attrs.remove(#attr_str); },
                    };
                }
            });
        } else {
            attr_accessors.push(quote! {
                pub fn #getter_name(&self) -> tir_core::Attr {
                    self.r#impl.attrs.get(#attr_str).unwrap().clone()
                }

                pub fn #setter_name<T>(&mut self, value: T) where tir_core::Attr: From<T> {
                    let attr = tir_core::Attr::from(value);
                    self.r#impl.attrs.insert(#attr_str.to_string(), attr);
                }
            });
        }
    }

    quote! {
        #(#attr_accessors)*
    }
}
