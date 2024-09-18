use crate::{OpFieldAttrs, OpFieldReceiver, OpReceiver};
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

fn make_operand_printer(fields: &[OpFieldReceiver]) -> proc_macro2::TokenStream {
    let mut printers = vec![];

    for f in fields {
        if let OpFieldAttrs::Operand = f.attrs {
            let operand_str = format!("{}", f.ident.as_ref().unwrap());
            let operand = f.ident.as_ref().unwrap();
            printers.push(quote! {
                fmt.write_direct(#operand_str);
                fmt.write_direct(" = ");
                self.#operand.print(fmt);
                fmt.write_direct(", ");
            });
        }
    }
    quote! {
        #(#printers)*
    }
}

fn make_return_type_printer(fields: &[OpFieldReceiver]) -> proc_macro2::TokenStream {
    for f in fields {
        if let OpFieldAttrs::Return = f.attrs {
            let ident = f.ident.as_ref().unwrap();
            return quote! {
                fmt.write_direct(" -> ");
                self.#ident.print(fmt);
            };
        }
    }

    quote! {}
}

fn make_return_type_parser(fields: &[OpFieldReceiver]) -> proc_macro2::TokenStream {
    for f in fields {
        if let OpFieldAttrs::Return = f.attrs {
            let ident = f.ident.as_ref().unwrap();
            return quote! {
                |builder| {
                    lpl::literal("->").spaced().and_then(tir_core::Type::parse).map(|(_, ty)| {
                        builder.#ident(ty);
                    })
                }
            };
        }
    }

    quote! {}
}

fn make_operands_parser(fields: &[OpFieldReceiver]) -> proc_macro2::TokenStream {
    let mut parsers = vec![
        quote! {
            lpl::literal("(").spaced()
        }
    ];

    parsers.extend(fields.iter().filter_map(|f| {
        if let OpFieldAttrs::Operand = f.attrs {
            let operand_str = format!("{}", f.ident.as_ref().unwrap());
            let ty = &f.ty;
            Some(quote!({
                .and_then(lpl::literal(#operand_str).and_then(lpl::literal("=").spaced()).map(|(_, _)| {
                    todo!()
                }))
            }))
        } else {
            None
        }
    }));

    parsers.push(quote! {
        lpl::literal(")").spaced()
    });

    quote! {
        |builder| {
            #(#parsers)*
        }
    }
}

pub fn make_generic_ir_printer_parser(op: DeriveInput) -> TokenStream {
    let op = OpReceiver::from_derive_input(&op).unwrap();
    let op_name = op.ident;

    let fields = op
        .data
        .take_struct()
        .unwrap()
        .into_iter()
        .collect::<Vec<OpFieldReceiver>>();

    let operand_printer = make_operand_printer(&fields);
    let return_printer = make_return_type_printer(&fields);
    let operands_parser = make_operands_parser(&fields);
    let return_parser = make_return_type_parser(&fields);

    quote! {
      impl tir_core::OpAssembly for #op_name {
          fn print_assembly(&self, fmt: &mut dyn tir_core::IRFormatter) {
            use tir_core::IRFormatter;
            #operand_printer

            fmt.write_direct("attrs = {");
            let attrs: Vec<_> = self
                .r#impl
                .attrs
                .iter()
                .map(|(name, attr)| {
                    let mut printer = tir_core::StringPrinter::new();
                    printer.write_direct(&format!("{} = ", name));
                    attr.print(&mut printer);
                    printer.get()
                })
                .collect();
            tir_core::print_comma_separated(fmt, &attrs);
            fmt.write_direct("}");

            #return_printer
          }

          fn parse_assembly<'a>() -> lpl::BoxedParser<'a, tir_core::parser::IRStrStream<'a>, tir_core::OpRef> {
            todo!()
            let operands_parser = #operands_parser;
            let mut builder = Self::builder(&input.state.get_context());

            let attr_list = tir_core::parser::attr_list.parse_next(input)?;

            #return_parser

            let op = builder.build();
            op.borrow_mut().add_attrs(&attr_list);

            Ok(op)
          }
      }
    }
}
