use quote::{format_ident, quote};
use std::{collections::HashMap, io::Write};

use crate::ast;

pub fn emit_rust<'a>(
    buf: &mut dyn Write,
    ast: &'a ast::SourceFile,
    dialect_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut items = HashMap::<String, &'a ast::Item>::new();
    let mut impls = HashMap::<String, Vec<&'a ast::Item>>::new();

    for i in ast.items() {
        match i {
            ast::Item::AsmDecl(decl) => {
                let name = decl.target_name();
                let vec = if impls.contains_key(&name) {
                    impls.get_mut(&name).unwrap()
                } else {
                    let vec = vec![];
                    impls.insert(name.clone(), vec);
                    impls.get_mut(&name).unwrap()
                };
                vec.push(i);
            }
            ast::Item::EncodingDecl(decl) => {
                let name = decl.target_name();
                let vec = if impls.contains_key(&name) {
                    impls.get_mut(&name).unwrap()
                } else {
                    let vec = vec![];
                    impls.insert(name.clone(), vec);
                    impls.get_mut(&name).unwrap()
                };
                vec.push(i);
            }
            ast::Item::ImplDecl(decl) => {
                let name = decl.target_name();
                let vec = if impls.contains_key(&name) {
                    impls.get_mut(&name).unwrap()
                } else {
                    let vec = vec![];
                    impls.insert(name.clone(), vec);
                    impls.get_mut(&name).unwrap()
                };
                vec.push(i);
            }
            _ => {
                items.insert(i.name(), i);
            }
        }
    }

    let rust_items: Vec<_> = ast
        .items()
        .iter()
        .filter_map(|item| match item {
            ast::Item::FlagDecl(ref flag) => Some(generate_flag(flag)),
            ast::Item::EnumDecl(ref enum_) => Some(generate_enum(&impls, enum_)),
            ast::Item::InstrDecl(ref instr) => Some(generate_instr(&items, instr, dialect_name)),
            _ => None,
        })
        .collect();

    let file: syn::File = syn::parse2(quote! { #(#rust_items)* }).unwrap();

    writeln!(buf, "{}", prettyplease::unparse(&file))?;

    Ok(())
}

fn generate_flag(decl: &ast::FlagDecl) -> proc_macro2::TokenStream {
    let name = format_ident!("{}", decl.name());
    let doc = if let Some(text) = decl.doc() {
        quote! { #[doc = #text] }
    } else {
        quote! {}
    };

    quote! {
        #doc
        pub struct #name;
    }
}

fn generate_enum<'a>(
    impls: &'a HashMap<String, Vec<&'a ast::Item>>,
    decl: &ast::EnumDecl,
) -> proc_macro2::TokenStream {
    let variants = decl.variants().iter().map(|v| {
        let ident = format_ident!("{}", v.name());
        let doc = if let Some(doc) = v.doc() {
            quote! { #[doc = #doc] }
        } else {
            quote! {}
        };

        quote! {
            #doc
            #ident
        }
    });

    let name = format_ident!("{}", decl.name());
    let doc = if let Some(doc) = decl.doc() {
        quote! { #[doc = #doc] }
    } else {
        quote! {}
    };

    let impl_tokens = impls.get(&decl.name()).into_iter().flatten().filter_map(
        |impl_| -> Option<proc_macro2::TokenStream> {
            if let ast::Item::ImplDecl(impl_) = impl_ {
                if impl_.trait_name() == "Register" {
                    let arms = decl.variants().iter().map(|v| {
                        let ident = format_ident!("{}", v.name());
                        let lowercase = v.name().to_lowercase();
                        quote! {
                            #name::#ident => fmt.write_direct(#lowercase)
                        }
                    });

                    return Some(quote! {
                        impl tir_core::Printable for #name {
                            fn print(&self, fmt: &mut dyn IRFormatter) {
                                match self {
                                    #(#arms),*
                                }
                            }
                        }
                    });
                }
            }

            None
        },
    );

    quote! {
        #doc
        pub enum #name {
            #(#variants),*
        }

        impl lpl::combinators::NotTuple for #name {}

        #(#impl_tokens)*
    }
}

fn generate_instr<'a>(
    other_decls: &'a HashMap<String, &'a ast::Item>,
    decl: &ast::InstrDecl,
    dialect_name: &str,
) -> proc_macro2::TokenStream {
    let mut fields = vec![];

    let mut parent = other_decls.get(&decl.template_name());

    while parent.is_some() {
        if let Some(ast::Item::InstrTemplateDecl(template)) = parent {
            for f in template.fields() {
                let name = format_ident!("{}", f.name());
                fields.push(quote! {
                    #[operand]
                    #name: Register
                });
            }

            parent = template
                .parent_template_name()
                .and_then(|n| other_decls.get(&n));
        } else {
            unreachable!("must have been an InstrTemplateDecl");
        }
    }

    let name = format_ident!("{}", decl.name());
    let dialect_name = format_ident!("{}", dialect_name);
    let op_name = decl.name().to_lowercase();

    quote! {
        #[derive(Op, OpAssembly, OpValidator)]
        #[operation(name = #op_name, dialect = #dialect_name)]
        pub struct #name {
            #(#fields),*,
            r#impl: OpImpl,
        }
    }
}
