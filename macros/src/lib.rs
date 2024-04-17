extern crate proc_macro;

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

    let mut attrs = vec![];
    let mut operands = vec![];
    let mut regions = vec![];

    for field in &input.fields {
        if field.attrs.len() != 1 {
            panic!("Expected all fields to have one attribute");
        }
        let op_field = match OperationField::from_field(field) {
            Ok(v) => v,
            Err(e) => {
                return TokenStream::from(e.write_errors());
            }
        };

        if op_field.attribute {
            attrs.push((op_field.ident.unwrap(), op_field.ty));
        } else if op_field.operand {
            operands.push(op_field.ident.unwrap());
        } else if op_field.region {
            regions.push((op_field.ident.unwrap(), op_field.single_block));
        }
    }

    let mut impls = vec![];

    for (attr, ty) in attrs {
        let attr_name = attr.to_string();
        let get_name = format_ident!("get_{}_attr", attr);
        let set_name = format_ident!("set_{}_attr", attr);

        let funcs = quote! {
            pub fn #get_name(&self) -> Attr {
                self.operation.borrow().attrs.get(#attr_name).unwrap().clone()
            }
            pub fn #set_name<T>(&mut self, value: T) where T: Into<#ty> {
                let tmp: #ty = value.into();
                self.operation.borrow_mut().attrs.insert(#attr_name.to_string(), Attr::from(tmp));
            }
        };
        impls.push(funcs);
    }

    for (id, (region, single_block)) in regions.iter().enumerate() {
        let get_func = format_ident!("get_{}_region", region);
        let func = quote! {
            pub fn #get_func(&self) -> Rc<RefCell<Region>> {
                self.operation
                    .borrow()
                    .regions[#id]
                    .clone()
            }
        };
        impls.push(func);

        if *single_block {
            let get_func = format_ident!("get_{}", region);
            let func = quote! {
                pub fn #get_func(&self) -> Rc<RefCell<Block>> {
                    self.operation
                        .borrow()
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

    for (id, operand) in operands.iter().enumerate() {
        let get_func = format_ident!("get_{}", operand);
        // TODO implement setters!
        let set_func = format_ident!("set_{}", operand);
        let funcs = quote! {
            pub fn #get_func(&self) -> Ref<'_, Operand> {
                Ref::map(self.operation.borrow(), |ops| &ops[#id])
            }
            pub fn #set_func(&mut self) {
                todo!()
            }
        };
        impls.push(funcs);
    }

    if regions.len() == 1 {
        let func = quote! {
            pub fn get_region(&self) -> Rc<RefCell<Region>> {
               self.operation.borrow().regions.first().unwrap().clone()
            }
        };
        impls.push(func);
    }

    let op_name_str = op_attrs.name;
    let op_name = input.ident;
    let traits = op_attrs.traits;

    TokenStream::from(quote! {
        #[derive(Debug)]
        pub struct #op_name {
            operation: Rc<RefCell<OperationImpl>>,
        }

        impl #op_name {
            #(#impls)*
        }

        impl Op for #op_name {
            fn get_operation_name() -> &'static str {
                #op_name_str
            }

            fn has_trait<T: ?Sized + 'static>() -> bool {
                let ids: Vec<TraitId> = vec![
                    #(trait_id::<dyn #traits>()),*
                ];
                ids.iter().find(|&x| x == &trait_id::<T>()).is_some()
            }
        }

        impl Into<Operation> for #op_name {
            fn into(self) -> Operation {
                Operation::from(self.operation.clone())
            }
        }

        impl TryFrom<Operation> for #op_name {
            type Error = ();

            fn try_from(operation: Operation) -> Result<Self, Self::Error> {
                if operation.get_operation_name() != Self::get_operation_name()
                    || operation.get_dialect_id()
                        != operation
                            .get_context()
                            .borrow()
                            .get_dialect_by_name(DIALECT_NAME)
                            .unwrap()
                            .borrow()
                            .get_id()
                {
                    return Err(());
                }

                Ok(Self { operation: operation.get_impl() })
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

        fn op_dispatcher(operation: Operation) -> Option<Box<dyn Op>> {
            None
        }

        pub fn create_dialect() -> Dialect {
            let mut dialect = Dialect::new(DIALECT_NAME, Box::new(op_dispatcher));

            populate_dialect_ops(&mut dialect);
            populate_dialect_types(&mut dialect);

            dialect
        }
    })
}

#[proc_macro]
pub fn populate_dialect_ops(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Types);

    let ty = input.0;

    TokenStream::from(quote! {
        fn populate_dialect_ops(dialect: &mut Dialect) {
            #(dialect.add_operation(#ty::get_operation_name());)*
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
