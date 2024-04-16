extern crate proc_macro;

use darling::ast::NestedMeta;
use darling::{Error, FromField, FromMeta};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

#[derive(FromMeta)]
struct OperationAttrs {
    pub name: String,
}

#[derive(FromField, Debug)]
#[darling(attributes(cfg))]
struct OperationField {
    pub ident: Option<syn::Ident>,
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

    let mut operands = vec![];
    let mut regions = vec![];

    for field in &input.fields {
        if field.attrs.len() != 1 {
            panic!("Expected all fields to have one attribute");
        }
        let op_field = match OperationField::from_field(&field) {
            Ok(v) => v,
            Err(e) => {
                return TokenStream::from(e.write_errors());
            }
        };

        if op_field.operand {
            operands.push(op_field.ident.unwrap());
        } else if op_field.region {
            regions.push((op_field.ident.unwrap(), op_field.single_block));
        }
    }

    let mut impls = vec![];

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
