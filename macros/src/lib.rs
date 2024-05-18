extern crate proc_macro;

mod assembly;
mod op_impl;

pub(crate) use assembly::*;
pub(crate) use op_impl::*;

use case_converter::camel_to_snake;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Token;
use syn::{parse_macro_input, LitStr};

#[derive(Debug)]
struct Types(Vec<syn::Ident>);

impl Parse for Types {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let result = Punctuated::<syn::Ident, Token![,]>::parse_terminated(input)?;

        Ok(Types(result.into_iter().collect()))
    }
}

struct DialectInput {
    name: syn::Ident,
    init: Option<syn::Expr>,
}

impl Parse for DialectInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        let init = match input.parse::<Token![,]>() {
            Ok(_) => Some(input.parse::<syn::Expr>()?),
            Err(_) => None,
        };

        Ok(Self{
            name,
            init
        })
    }
}

#[proc_macro]
pub fn dialect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DialectInput);

    let dialect_name = input.name.to_string();
    // let name_ident = parse_macro_input!(input as syn::Ident);
    // let dialect_name = name_ident.to_string();
    
    let init = match input.init {
        Some(init) => quote! {
            let init = #init;
            init(&mut dialect);
        },
        _ => quote! {}
    };

    TokenStream::from(quote! {
        pub const DIALECT_NAME: &str = #dialect_name;

        pub fn create_dialect() -> Dialect {
            let mut dialect = Dialect::new(DIALECT_NAME);

            populate_dialect_ops(&mut dialect);
            populate_dialect_types(&mut dialect);

            #init

            dialect
        }
    })
}

#[proc_macro]
pub fn dialect_type(input: TokenStream) -> TokenStream {
    let name_ident = parse_macro_input!(input as syn::Ident);
    let name_string = name_ident.to_string();
    let name_str = name_string.strip_suffix("Type").unwrap_or(&name_string);
    let name_str = &camel_to_snake(name_str)[1..];

    quote! {
        #[derive(Clone)]
        pub struct #name_ident {
            r#type: Type,
        }

        impl tir_core::Ty for #name_ident {
            fn get_type_name() -> &'static str {
                #name_str
            }

            fn get_dialect_name() -> &'static str {
                DIALECT_NAME
            }
        }

        impl tir_core::TyAssembly for #name_ident {
            fn print_assembly(attrs: &HashMap<String, tir_core::Attr>, fmt: &mut dyn tir_core::IRFormatter) {
                // FIXME: make attrs optional
                fmt.write_direct(#name_str);
                fmt.write_direct(" ");
                fmt.write_direct("attrs = {");
                for (name, attr) in attrs {
                    fmt.write_direct(name);
                    fmt.write_direct(" = ");
                    attr.print(fmt);
                }
                fmt.write_direct("}");
            }

            fn parse_assembly(input: &mut tir_core::parser::ParseStream<'_>) -> tir_core::parser::AsmPResult<std::collections::HashMap<String, tir_core::Attr>> {
                // FIXME: make attrs optional
                tir_core::parser::attr_list(input)
            }
        }

        impl tir_core::Printable for #name_ident {
            fn print(&self, fmt: &mut dyn crate::IRFormatter) {
                fmt.write_direct("!");
                if DIALECT_NAME != tir_core::builtin::DIALECT_NAME {
                    fmt.write_direct(&format!("{}.", DIALECT_NAME));
                }

                Self::print_assembly(self.r#type.get_attrs(), fmt);
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
                    // we are sure the type exists, because we are the type!
                    let type_id = dialect.get_type_id(#name_ident::get_type_name()).unwrap();
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

        impl TryFrom<Type> for #name_ident {
            type Error = ();

            fn try_from(ty: Type) -> Result<Self, Self::Error> {
                if !ty.isa::<#name_ident>() {
                    return Err(());
                }

                Ok(Self { r#type: ty })
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
            #(dialect.add_operation(#ty::get_operation_name(), <#ty>::parse_assembly);)*
        }
    })
}

#[proc_macro]
pub fn populate_dialect_types(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Types);

    let ty = input.0;

    TokenStream::from(quote! {
        fn populate_dialect_types(dialect: &mut Dialect) {
            #(dialect.add_type(#ty::get_type_name(), #ty::print_assembly, #ty::parse_assembly);)*
        }
    })
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
}

fn build_return_type_accessor(fields: &[OpFieldReceiver]) -> proc_macro2::TokenStream {
    for field in fields {
        if let OpFieldAttrs::Return = field.attrs {
            let ident = field.ident.clone().unwrap();
            return quote! {
                fn get_return_type(&self) -> Option<tir_core::Type> {
                    Some(self.#ident.clone())
                }

                fn get_return_value(&self) -> Option<tir_core::Value> {
                    let context = self.get_context();
                    Some(tir_core::Value::from_op(context, "todo", self.r#impl.alloc_id))
                }
            };
        }
    }

    quote! {
        fn get_return_type(&self) -> Option<tir_core::Type> {
            None
        }

        fn get_return_value(&self) -> Option<tir_core::Value> {
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

    let attr_accessors = if !attrs.is_empty() {
        build_attr_accessors(&attrs)
    } else {
        quote! {}
    };

    quote! {
        impl tir_core::Printable for #op_ident {
            fn print(&self, fmt: &mut dyn tir_core::IRFormatter) where Self: tir_core::OpAssembly {
                fmt.indent();
                if DIALECT_NAME != tir_core::builtin::DIALECT_NAME {
                    fmt.write_direct(DIALECT_NAME);
                    fmt.write_direct(".");
                }

                fmt.write_direct(self.get_operation_name());
                fmt.write_direct(" ");

                self.print_assembly(fmt);
                fmt.write_direct("\n");
            }
        }

        impl tir_core::Op for #op_ident {
            fn get_operation_name(&self) -> &'static str {
                #name
            }

            fn get_attrs(&self) -> &std::collections::HashMap<String, tir_core::Attr> {
                todo!();
            }

            fn add_attrs(&mut self, attrs: &std::collections::HashMap<String, tir_core::Attr>) {
                for (k, v) in attrs {
                    self.r#impl.attrs.insert(k.clone(), v.clone());
                }
            }

            fn get_context(&self) -> tir_core::ContextRef {
                let context = self.r#impl.context.upgrade();
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

#[proc_macro_derive(OpAssembly)]
pub fn derive_op_assembly(input: TokenStream) -> TokenStream {
    let op = parse_macro_input!(input as syn::DeriveInput);
    make_generic_ir_printer_parser(op).into()
}

#[proc_macro]
pub fn lowercase(input: TokenStream) -> TokenStream {
    let literal = if let Ok(literal) = syn::parse::<LitStr>(input.clone()) {
        literal.value()
    } else {
        let input = parse_macro_input!(input as syn::Ident);
        format!("{}", input)
    };

    let res = literal.to_lowercase();

    quote! {
        #res
    }
    .into()
}

#[proc_macro]
pub fn uppercase(input: TokenStream) -> TokenStream {
    let literal = if let Ok(literal) = syn::parse::<LitStr>(input.clone()) {
        literal.value()
    } else {
        let input = parse_macro_input!(input as syn::Ident);
        format!("{}", input)
    };

    let res = literal.to_uppercase();

    quote! {
        #res
    }
    .into()
}
