extern crate proc_macro;

mod assembly;

use assembly::*;
use case_converter::camel_to_snake;
use darling::{FromDeriveInput, FromField, FromMeta};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Parser};
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::Token;

#[derive(Debug)]
struct Types(Vec<syn::Ident>);

impl Parse for Types {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let result = Punctuated::<syn::Ident, Token![,]>::parse_terminated(input)?;

        Ok(Types(result.into_iter().collect()))
    }
}

#[proc_macro]
pub fn dialect(input: TokenStream) -> TokenStream {
    let name_ident = parse_macro_input!(input as syn::Ident);
    let dialect_name = name_ident.to_string();

    TokenStream::from(quote! {
        pub(crate) const DIALECT_NAME: &str = #dialect_name;

        pub fn create_dialect() -> Dialect {
            let mut dialect = Dialect::new(DIALECT_NAME);

            populate_dialect_ops(&mut dialect);
            populate_dialect_types(&mut dialect);

            dialect
        }
    })
}

#[proc_macro]
pub fn dialect_type(input: TokenStream) -> TokenStream {
    let name_ident = parse_macro_input!(input as syn::Ident);
    let name_string = name_ident.to_string();
    let name_str = name_string.strip_suffix("Type").unwrap_or(&name_string);
    let name_str = camel_to_snake(name_str);

    quote! {
        pub struct #name_ident {
            r#type: Type,
        }

        impl tir_core::Ty for #name_ident {
            fn get_type_name() -> &'static str {
                #name_str
            }
        }

        impl Into<Attr> for #name_ident {
            fn into(self) -> Attr {
                Attr::Type(self.r#type)
            }
        }

        impl Into<Type> for #name_ident {
            fn into(self) -> Type {
                self.r#type
            }
        }

        impl TryFrom<Attr> for #name_ident {
            type Error = ();

            fn try_from(attr: Attr) -> Result<Self, Self::Error> {
                if let Attr::Type(ty) = attr {
                    let context = ty.get_context().ok_or(())?;
                    let dialect = context.get_dialect_by_name(DIALECT_NAME).unwrap();
                    let type_id = dialect.get_type_id(#name_ident::get_type_name());
                    if type_id != ty.get_type_id() {
                        return Err(());
                    }

                    return Ok(#name_ident {
                        r#type: ty,
                    });
                }
                Err(())
            }
        }
    }
    .into()
}

#[proc_macro]
pub fn populate_dialect_ops(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Types);

    let ty = input.0;

    TokenStream::from(quote! {
        fn populate_dialect_ops(dialect: &mut Dialect) {
            #(dialect.add_operation(#ty::get_operation_name(), <#ty>::parse);)*
        }
    })
}

#[proc_macro]
pub fn populate_dialect_types(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Types);

    let ty = input.0;

    TokenStream::from(quote! {
        fn populate_dialect_types(dialect: &mut Dialect) {
            #(dialect.add_type(#ty::get_type_name());)*
        }
    })
}

#[derive(Debug)]
struct OpAttrs {
    attrs: Vec<Attr>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Attr(syn::Ident, syn::Type);

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
struct OpReceiver {
    ident: syn::Ident,
    data: darling::ast::Data<(), OpFieldReceiver>,
    name: String,
    #[darling(default)]
    known_attrs: Option<OpAttrs>,
}

#[derive(Default, Debug, FromMeta)]
struct RegionAttrs {
    #[darling(default)]
    single_block: bool,
    #[darling(default)]
    no_args: bool,
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
enum OpFieldAttrs {
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
struct OpFieldReceiver {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(with = transform_field_attrs)]
    attrs: OpFieldAttrs,
}

fn build_operand_accessors(fields: &[OpFieldReceiver]) -> proc_macro2::TokenStream {
    let mut accessors = vec![];

    for field in fields {
        if let OpFieldAttrs::Operand = &field.attrs {
            let ident = field.ident.clone().unwrap();
            let ty = field.ty.clone();

            let get_name = format_ident!("get_{}", ident);
            let set_name = format_ident!("set_{}", ident);

            accessors.push(quote! {
                pub fn #get_name(&self) -> #ty {
                    self.#ident.clone()
                }
                pub fn #set_name(&mut self, value: #ty) {
                    self.#ident = value
                }
            });
        }
    }

    quote! {
        #(#accessors)*
    }
}

fn build_region_accessors(fields: &[OpFieldReceiver]) -> proc_macro2::TokenStream {
    let mut accessors = vec![];

    for field in fields {
        if let OpFieldAttrs::Region(region) = &field.attrs {
            let ident = field.ident.clone().unwrap();
            let ty = field.ty.clone();

            let get_name = format_ident!("get_{}_region", ident);

            accessors.push(quote! {
                pub fn #get_name(&self) -> #ty {
                    self.#ident.clone()
                }
            });

            if region.single_block {
                let get_name = format_ident!("get_{}", ident);
                accessors.push(quote! {
                    pub fn #get_name(&self) -> tir_core::BlockRef {
                        self.#ident.first().unwrap()
                    }
                });
            }
        }
    }

    quote! {
        #(#accessors)*
    }
}

fn build_attr_accessors(attrs: &[Attr]) -> proc_macro2::TokenStream {
    let mut attr_accessors = vec![];

    for attr in attrs {
        let getter_name = format_ident!("get_{}_attr", attr.0);
        let setter_name = format_ident!("set_{}_attr", attr.0);
        let attr_str = attr.0.to_string();

        attr_accessors.push(quote! {
            pub fn #getter_name<'a>(&'a self) -> &'a tir_core::Attr {
                self.r#impl.attrs.get(#attr_str).unwrap()
            }

            pub fn #setter_name<T>(&mut self, value: T) where tir_core::Attr: From<T> {
                let attr = tir_core::Attr::from(value);
                self.r#impl.attrs.insert(#attr_str.to_string(), attr);
            }
        });
    }

    quote! {
        #(#attr_accessors)*
    }
    .into()
}

fn build_return_type_accessor(fields: &[OpFieldReceiver]) -> proc_macro2::TokenStream {
    for field in fields {
        if let OpFieldAttrs::Return = field.attrs {
            let ident = field.ident.clone().unwrap();
            return quote! {
                fn get_return_type(&self) -> Option<tir_core::Type> {
                    Some(self.#ident.clone())
                }
            };
        }
    }

    quote! {
        fn get_return_type(&self) -> Option<tir_core::Type> {
            None
        }
    }
}

fn build_op_builder(
    op: syn::Ident,
    op_name: &str,
    fields: &[OpFieldReceiver],
    attrs: &[Attr],
) -> proc_macro2::TokenStream {
    let builder_name = format_ident!("{}Builder", op);

    let mut builder_fields = vec![];
    let mut builder_accessors = vec![];
    let mut builder_setters = vec![];
    let mut field_idents = vec![];
    let mut attr_setters = vec![];

    for attr in attrs {
        let ident = &attr.0;
        builder_fields.push(quote! {
            #ident: Option<tir_core::Attr>,
        });

        builder_accessors.push(quote! {
            pub fn #ident(mut self, value: tir_core::Attr) -> Self {
                self.#ident = Some(value);
                self
            }
        });

        builder_setters.push(quote! {
            #ident: None,
        });

        let attr_str = format!("{}", ident);
        attr_setters.push(quote! {
            if let Some(attr) = &self.#ident {
                attrs.insert(#attr_str.to_string(), attr.clone());
            }
        });
    }

    for field in fields {
        if let OpFieldAttrs::None = &field.attrs {
            continue;
        }

        field_idents.push(field.ident.clone());
        let name = &field.ident;
        let ty = &field.ty;

        builder_fields.push(quote! {
            #name: Option<#ty>,
        });

        builder_accessors.push(quote! {
            pub fn #name(mut self, value: #ty) -> Self {
                self.#name = Some(value);
                self
            }
        });

        let mut add_empty_setter = || {
            builder_setters.push(quote! {
                #name: None,
            });
        };

        match &field.attrs {
            OpFieldAttrs::Region(region) => {
                if region.single_block && region.no_args {
                    builder_setters.push(quote! {
                        #name: Some(tir_core::Region::with_single_block(&context)),
                    });
                } else {
                    add_empty_setter()
                }
            }
            _ => add_empty_setter(),
        }
    }

    quote! {
        pub struct #builder_name {
            context: tir_core::ContextRef,
            #(#builder_fields)*
        }

        impl #op {
            pub fn builder(context: &tir_core::ContextRef) -> #builder_name {
                #builder_name {
                    context: context.clone(),
                    #(#builder_setters)*
                }
            }
        }


        impl #builder_name {
            #(#builder_accessors)*

            pub fn build(self) -> std::rc::Rc<std::cell::RefCell<#op>> {
                let context = self.context.clone();
                let dialect = context.get_dialect_by_name(DIALECT_NAME).unwrap();
                let dialect_id = dialect.get_id();
                let operation_id = dialect.get_operation_id(#op_name).expect("We just registered the operation");
                let mut attrs = std::collections::HashMap::new();

                #(#attr_setters)*

                let weak = std::sync::Arc::downgrade(&context).clone();

                let r#impl = tir_core::OpImpl {
                    context: std::sync::Arc::downgrade(&context),
                    dialect_id,
                    operation_id,
                    alloc_id: tir_core::AllocId::default(),
                    parent_region: None,
                    attrs,
                };

                let operation = #op {
                    #(#field_idents: self.#field_idents.unwrap(),)*
                    r#impl,
                };
                
                context.allocate_op(operation)
            }
        }
    }
}

#[proc_macro_derive(Op, attributes(operation, operand, region, ret_type))]
pub fn derive_op(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let op = OpReceiver::from_derive_input(&input).unwrap();

    let fields = op
        .data
        .take_struct()
        .unwrap()
        .into_iter()
        .collect::<Vec<OpFieldReceiver>>();

    let region_accessors = build_region_accessors(&fields);
    let operand_accessors = build_operand_accessors(&fields);
    let return_type = build_return_type_accessor(&fields);

    let op_ident = op.ident;
    let name = op.name;

    let attrs = if let Some(attrs) = op.known_attrs {
        attrs.attrs.clone()
    } else {
        vec![]
    };

    let builder = build_op_builder(op_ident.clone(), &name, &fields, &attrs);

    let attr_accessors = if attrs.len() > 0 {
        build_attr_accessors(&attrs)
    } else {
        quote! {}
    };

    quote! {
        impl Op for #op_ident {
            fn get_operation_name(&self) -> &'static str {
                #name
            }

            fn get_attrs(&self) -> &std::collections::HashMap<String, tir_core::Attr> {
                todo!();
            }

            fn get_context(&self) -> tir_core::ContextRef {
                // eprintln!("{:?}", self);
                let context = self.r#impl.context.upgrade();
                eprintln!("{:?}", context);
                self.r#impl.context.upgrade().unwrap()
            }

            fn get_parent_region(&self) -> Option<tir_core::RegionRef> {
                self.r#impl.parent_region.clone().map(|r| r.upgrade())?
            }

            fn set_alloc_id(&mut self, id: tir_core::AllocId) {
                assert_eq!(self.r#impl.alloc_id, tir_core::AllocId::default());
                assert_ne!(id, tir_core::AllocId::default());
                self.r#impl.alloc_id = id;
            }

            fn get_alloc_id(&self) -> tir_core::AllocId {
                assert_ne!(self.r#impl.alloc_id, tir_core::AllocId::default());
                self.r#impl.alloc_id
            }

            fn get_dialect_id(&self) -> u32 {
                self.r#impl.dialect_id
            }

            #return_type
        }

        impl #op_ident {
            #region_accessors
            #operand_accessors
            #attr_accessors

            pub fn get_operation_name() -> &'static str {
                #name
            }
        }

        #builder
    }
    .into()
}

#[proc_macro_derive(Assembly)]
pub fn derive_assembly(input: TokenStream) -> TokenStream {
    let op = parse_macro_input!(input as syn::DeriveInput);
    make_generic_ir_printer_parser(op.ident).into()
}
