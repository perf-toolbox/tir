use quote::{format_ident, quote};
use std::{collections::HashMap, io::Write};

use crate::ast::{self, AttrListOwner};

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
                    let names_arms = decl.variants().iter().map(|v| {
                        let attr_list = v.attr_list().unwrap().clone();
                        let list_expr = attr_list
                            .attributes()
                            .find(|v| v.name() == "reg_names")
                            .unwrap()
                            .exprs()
                            .next()
                            .unwrap()
                            .as_list();
                        let names = list_expr.elements().iter().map(|el| {
                            let text = el.as_literal().text();
                            text[1..text.len() - 1].to_string()
                        });
                        let ident = format_ident!("{}", v.name());
                        quote! {
                            #name::#ident => &[#(#names),*]
                        }
                    });

                    let parser_arms = decl.variants().iter().map(|v| {
                        let attr_list = v.attr_list().unwrap().clone();
                        let list_expr = attr_list
                            .attributes()
                            .find(|v| v.name() == "reg_names")
                            .unwrap()
                            .exprs()
                            .next()
                            .unwrap()
                            .as_list();
                        let names = list_expr.elements().iter().map(|el| {
                            let text = el.as_literal().text();
                            text[1..text.len() - 1].to_string()
                        });

                        let ident = format_ident!("{}", v.name());
                        quote! {
                            #(#names)|* => Some(#name::#ident)
                        }
                    });

                    let to_num_arms = decl.variants().iter().enumerate().map(|(id, v)| {
                        let ident = format_ident!("{}", v.name());

                        quote! {
                            #name::#ident => #id
                        }
                    });

                    let from_num_arms = decl.variants().iter().enumerate().map(|(id, v)| {
                        let ident = format_ident!("{}", v.name());

                        quote! {
                            #id => #name::#ident
                        }
                    });

                    let parser_name = format_ident!("parse_{}", name.to_string().to_lowercase());

                    return Some(quote! {
                        impl #name {
                            pub fn get_names(&self) -> &'static [&'static str] {
                                match self {
                                    #(#names_arms),*
                                }
                            }
                            pub fn get_reg_num(&self) -> usize {
                                match &self {
                                    #(#to_num_arms),*
                                }
                            }

                            pub fn encode(&self) -> usize {

                            }
                        }
                        impl tir_core::Printable for #name {
                            fn print(&self, fmt: &mut dyn tir_core::IRFormatter) {
                                fmt.write_direct(self.get_names()[0])
                            }
                        }

                        #[allow(clippy::from_over_into)]
                        impl Into<tir_core::Attr> for #name {
                            fn into(self) -> tir_core::Attr {
                                tir_core::Attr::String(self.get_reg_names()[0].to_string())
                            }
                        }

                        pub fn #parser_name(input: &str) -> Option<#name> {
                            match self {
                                #(#parser_arms),*,
                                _ => None,
                            }
                        }
                        impl tir_core::Parsable<#name> for #name {
                            fn parse(input: tir_core::IRStrStream) -> lpl::ParseResult<tir_core::IRStrStream, #name> {
                                let parser = ident(|_| false).try_map(|r, s| {
                                    #parser_name(r).ok_or(Into::<lpl::Diagnostic>::into(DiagKind::UnknownRegister(
                                        r.to_string(),
                                        s,
                                    )))
                                });
                                parser.parse(input)
                            }
                        }

                        impl TryFrom<usize> for #name {
                            type Error = ();
                            fn try_from(value: usize) -> Result<Self, Self::Error> {
                                match value {
                                    #(#from_num_arms),*,
                                    _ => Err(())
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
