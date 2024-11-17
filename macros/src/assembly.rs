use crate::{OpFieldAttrs, OpFieldReceiver, OpReceiver};
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
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
                let return_parser =
                    lpl::combinators::literal("->").spaced().and_then(tir_core::Type::parse).map(|(_, ty): (_, tir_core::Type)| {
                        ty
                    });

                let (ty, next_input) = return_parser.parse(next_input.unwrap())?;

                let builder = builder.#ident(ty);
            };
        }
    }

    quote! {}
}

fn make_operands_parser(fields: &[OpFieldReceiver]) -> proc_macro2::TokenStream {
    let mut parsers = vec![quote! {
        lpl::combinators::any_whitespace0()
    }];

    let mut names = vec![];

    parsers.extend(fields.iter().filter_map(|f| {
        if let OpFieldAttrs::Operand = f.attrs {
            let operand_str = format!("{}", f.ident.as_ref().unwrap());
            let ident = f.ident.as_ref().unwrap();
            names.push(ident);
            let span = ident.span();
            let ty = &f.ty;
            Some(quote_spanned! {span=>
                .and_then(
                    lpl::combinators::literal(#operand_str)
                        .and_then(lpl::combinators::spaced(lpl::combinators::literal("=")))
                        .and_then(#ty::parse)
                        .and_then(lpl::combinators::spaced(lpl::combinators::literal(",")))
                        .flat().map(|(_, _, value, _)| {
                            value
                        }))
            })
        } else {
            None
        }
    }));

    parsers.push(quote! {
        .flat()
    });

    let mut builder = vec![];

    for n in &names {
        let span = n.span();
        builder.push(quote_spanned! {span=>
            builder = builder.#n(#n);
        })
    }

    if parsers.len() == 2 {
        return quote! {};
    }

    let operands_parser = quote! {
        #(#parsers)*
    };

    quote! {
        let operands_parser = #operands_parser;
        let ((_, #(#names),*), next_input) = operands_parser.parse(next_input.unwrap())?;
        #(#builder)*
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

          fn parse_assembly<'a>(input: tir_core::IRStrStream<'a>) -> lpl::ParseResult<tir_core::IRStrStream<'a>, tir_core::OpRef> {
            use tir_core::parser::Parsable;
            let state = input.get_extra().unwrap();
            let context = state.context();
            let mut builder = Self::builder(&context);
            let attrs_parser = tir_core::parser::attr_list();

            let next_input = Some(input);

            #operands_parser

            let (attr_list, next_input) = attrs_parser.parse(next_input.unwrap())?;

            #return_parser

            let op = builder.build();
            op.borrow_mut().add_attrs(&attr_list);
            let op: tir_core::OpRef = op;

            Ok((op, next_input))
          }
      }
    }
}
