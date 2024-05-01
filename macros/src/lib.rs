extern crate proc_macro;

mod assembly;

use assembly::*;
use case_converter::camel_to_snake;
use darling::ast::NestedMeta;
use darling::{Error, FromField, FromMeta};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::Token;

#[derive(FromMeta)]
struct OperationAttrs {
    pub name: String,
    #[darling(default)]
    pub traits: darling::util::PathList,
    #[darling(default)]
    pub custom_assembly: bool,
}

#[derive(FromField, Debug)]
#[darling(attributes(cfg))]
struct OperationField {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
    #[darling(default)]
    pub attribute: bool,
    #[darling(default)]
    pub operand: bool,
    #[darling(default)]
    pub region: bool,
    #[darling(default)]
    pub single_block: bool,
}

struct AttrField {
    ident: syn::Ident,
    ty: syn::Type,
}

struct OperandField {
    ident: syn::Ident,
    _ty: syn::Type,
}

struct RegionField {
    ident: syn::Ident,
    single_block: bool,
}

fn parse_fields(
    fields: &syn::Fields,
) -> Result<(Vec<AttrField>, Vec<OperandField>, Vec<RegionField>), darling::Error> {
    let mut attrs = vec![];
    let mut operands = vec![];
    let mut regions = vec![];

    for field in fields {
        if field.attrs.len() != 1 {
            panic!("Expected all fields to have one attribute");
        }
        let op_field = OperationField::from_field(field)?;

        if op_field.attribute {
            attrs.push(AttrField {
                ident: op_field.ident.unwrap(),
                ty: op_field.ty,
            });
        } else if op_field.operand {
            operands.push(OperandField {
                ident: op_field.ident.unwrap(),
                _ty: op_field.ty,
            });
        } else if op_field.region {
            regions.push(RegionField {
                ident: op_field.ident.unwrap(),
                single_block: op_field.single_block,
            });
        }
    }

    Ok((attrs, operands, regions))
}

fn build_attr_accessors(attrs: &[AttrField]) -> proc_macro2::TokenStream {
    let mut impls = vec![];

    for attr in attrs {
        let attr_name = attr.ident.to_string();
        let ty = &attr.ty;
        let get_name = format_ident!("get_{}_attr", attr.ident);
        let set_name = format_ident!("set_{}_attr", attr.ident);

        let funcs = quote! {
            pub fn #get_name(&self) -> Attr {
                self.operation.attrs.get(#attr_name).unwrap().clone()
            }
            pub fn #set_name<T>(&mut self, value: T) where T: Into<#ty> {
                let tmp: #ty = value.into();
                self.operation.attrs.insert(#attr_name.to_string(), tmp.into());
            }
        };

        impls.push(funcs);
    }

    quote! {
        #(#impls)*
    }
}

fn build_operand_accessors(operands: &[OperandField]) -> proc_macro2::TokenStream {
    let mut impls = vec![];

    for (id, operand) in operands.iter().enumerate() {
        let get_func = format_ident!("get_{}", operand.ident);
        // TODO implement setters!
        let set_func = format_ident!("set_{}", operand.ident);
        let funcs = quote! {
            pub fn #get_func(&self) -> &Operand {
                &self.operation.operands[#id]
            }
            pub fn #set_func(&mut self) {
                todo!()
            }
        };

        impls.push(funcs);
    }

    quote! {
        #(#impls)*
    }
}

fn build_region_accessors(regions: &[RegionField]) -> proc_macro2::TokenStream {
    let mut impls = vec![];

    for (id, region) in regions.iter().enumerate() {
        let get_func = format_ident!("get_{}_region", region.ident);
        let func = quote! {
            pub fn #get_func(&self) -> RegionRef {
                self.operation
                    .regions[#id]
                    .clone()
            }
        };
        impls.push(func);

        if region.single_block {
            let get_func = format_ident!("get_{}", region.ident);
            let func = quote! {
                pub fn #get_func(&self) -> BlockRef {
                    self.operation
                        .regions[#id]
                        .borrow()
                        .get_blocks()
                        .first()
                        .unwrap()
                        .clone()
                }
            };
            impls.push(func);
        }
    }

    if regions.len() == 1 {
        let func = quote! {
            pub fn get_region(&self) -> RegionRef {
               self.operation.regions.first().unwrap().clone()
            }
        };
        impls.push(func);
    }

    quote! {
        #(#impls)*
    }
}

fn build_op_builder(
    op: &syn::Ident,
    op_name: &str,
    attrs: &[AttrField],
    operands: &[OperandField],
    regions: &[RegionField],
) -> proc_macro2::TokenStream {
    let builder_name = format_ident!("{}Builder", op);

    let mut builder_fields = vec![];
    let mut builder_accessors = vec![];
    let mut builder_setters = vec![];
    let mut builder_fillers = vec![];

    for attr in attrs {
        let name = &attr.ident;

        builder_fields.push(quote! {
            #name: Option<Attr>,
        });

        builder_accessors.push(quote! {
            pub fn #name(mut self, attr: Attr) -> Self {
                self.#name = Some(attr);
                self
            }
        });

        builder_setters.push(quote! {
            #name: None,
        });

        let name_str = name.to_string();
        builder_fillers.push(quote! {
            if let Some(val) = self.#name {
                attrs.insert(#name_str.to_string(), val);
            }
        });
    }

    for operand in operands {
        let name = &operand.ident;

        builder_fields.push(quote! {
            #name: Option<Operand>,
        });

        builder_accessors.push(quote! {
            pub fn #name(mut self, operand: Operand) -> Self {
                self.#name = Some(operand);
                self
            }
        });

        builder_setters.push(quote! {
            #name: None,
        });

        builder_fillers.push(quote! {
            if let Some(val) = self.#name {
               operands.push(val);
            }
        });
    }

    for region in regions {
        builder_fillers.push(quote! {
            let region = Region::new(context.clone());
            regions.push(region.clone());
        });

        if region.single_block {
            builder_fillers.push(quote! {
                region.borrow_mut().emplace_block(Rc::downgrade(&region));
            });
        }
    }

    quote! {
        pub struct #builder_name {
            context: ContextRef,
            #(#builder_fields)*
        }

        impl #op {
            pub fn builder(context: ContextRef) -> #builder_name {
                #builder_name {
                    context,
                    #(#builder_setters)*
                }
            }
        }

        impl #builder_name {
            #(#builder_accessors)*

            pub fn build(self) -> std::rc::Rc<std::cell::RefCell<#op>> {
                let context = self.context.clone();
                let dialect = context.borrow().get_dialect_by_name(DIALECT_NAME).unwrap();
                let dialect_id = dialect.borrow().get_id();
                let operation_id = dialect.borrow().get_operation_id(#op_name).expect("We just registered the operation");
                let mut attrs = std::collections::HashMap::new();
                let mut operands = vec![];
                let mut regions = vec![];

                #(#builder_fillers)*

                let operation = Rc::new(RefCell::new(#op{operation: OperationImpl{
                    context: self.context.clone(),
                    dialect_id,
                    operation_id,
                    operands,
                    attrs,
                    regions,
                }}));

                operation
            }
        }
    }
}

#[proc_macro_attribute]
pub fn operation(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let args = match NestedMeta::parse_meta_list(metadata.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };
    let op_attrs = match OperationAttrs::from_list(&args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let input = parse_macro_input!(input as syn::ItemStruct);

    let (attrs, operands, regions) = match parse_fields(&input.fields) {
        Ok((attrs, operands, regions)) => (attrs, operands, regions),
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let attr_accessors = build_attr_accessors(&attrs);
    let operand_accessors = build_operand_accessors(&operands);
    let region_accessors = build_region_accessors(&regions);

    let op_builder = build_op_builder(&input.ident, &op_attrs.name, &attrs, &operands, &regions);

    let op_name_str = op_attrs.name;
    let op_name = input.ident;
    let traits = op_attrs.traits;

    let printer = if !op_attrs.custom_assembly {
        make_generic_ir_printer_parser(op_name.clone())
    } else {
        quote! {}
    };

    TokenStream::from(quote! {
        pub struct #op_name {
            operation: OperationImpl,
        }

        #printer

        #op_builder

        impl #op_name {
            #attr_accessors
            #operand_accessors
            #region_accessors
        }

        impl Op for #op_name {
            fn get_operation_name() -> &'static str {
                #op_name_str
            }

            fn get_op_name(&self) -> &'static str {
                #op_name_str
            }

            fn has_trait<T: ?Sized + 'static>() -> bool {
                let ids: Vec<TraitId> = vec![
                    #(trait_id::<dyn #traits>()),*
                ];
                ids.iter().find(|&x| x == &trait_id::<T>()).is_some()
            }

            fn get_context(&self) -> ContextRef {
                self.operation.context.clone()
            }

            fn get_dialect_id(&self) -> u32 {
                self.operation.dialect_id
            }

            fn emplace_region(&mut self) -> RegionRef {
                let region = Region::new(self.get_context());
                self.operation.regions.push(region.clone());
                region
            }

            fn get_regions(&self) -> &[RegionRef] {
                &self.operation.regions
            }

            fn add_attr(&mut self, name: String, attr: Attr) {
                self.operation.attrs.insert(name, attr);
            }

            fn get_attrs(&self) -> &std::collections::HashMap<String, Attr> {
                &self.operation.attrs
            }

            fn add_operand(&mut self, operand: Operand) {
                self.operation.operands.push(operand);
            }
        }
    })
}

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

        impl Ty for #name_ident {
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
                    let context = ty.get_context();
                    let dialect = context.borrow().get_dialect_by_name(DIALECT_NAME).unwrap();
                    let type_id = dialect.borrow().get_type_id(#name_ident::get_type_name());
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
