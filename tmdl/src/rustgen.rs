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
            _ => {
                items.insert(i.name(), i);
            }
        }
    }

    for item in ast.items() {
        match item {
            ast::Item::EnumDecl(ref enum_) => print_enum(buf, enum_)?,
            ast::Item::InstrDecl(ref instr) => print_instr(buf, &items, instr, dialect_name)?,
            _ => {}
        }
    }

    Ok(())
}

fn print_enum(buf: &mut dyn Write, decl: &ast::EnumDecl) -> Result<(), std::io::Error> {
    writeln!(buf, "pub enum {} {{", decl.name())?;

    for var in decl.variants() {
        writeln!(buf, "    {},", var.name())?;
    }

    writeln!(buf, "}}\n")?;

    Ok(())
}

fn print_instr<'a>(
    buf: &mut dyn Write,
    other_decls: &'a HashMap<String, &'a ast::Item>,
    decl: &ast::InstrDecl,
    dialect_name: &str,
) -> Result<(), std::io::Error> {
    let mut struct_fields: Vec<&'a ast::StructFieldDecl> = vec![];

    let mut parent = other_decls.get(&decl.template_name());

    while parent.is_some() {
        if let Some(ast::Item::InstrTemplateDecl(template)) = parent {
            for f in template.fields() {
                struct_fields.push(f);
            }

            parent = template
                .parent_template_name()
                .and_then(|n| other_decls.get(&n));
        } else {
            unreachable!("must have been an InstrTemplateDecl");
        }
    }

    writeln!(buf, "#[derive(Op, OpAssembly, OpValidator)]")?;
    writeln!(
        buf,
        "#[operation(name = \"{}\", dialect = {dialect_name})]",
        decl.name().to_lowercase()
    )?;
    writeln!(buf, "pub struct {}Op {{", decl.name())?;

    for f in struct_fields {
        writeln!(buf, "    {}: bool,", f.name())?;
    }

    writeln!(buf, "    r#impl: OpImpl,")?;
    writeln!(buf, "}}\n")?;

    Ok(())
}
