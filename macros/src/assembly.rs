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
                let ty = winnow::combinator::preceded((winnow::ascii::space0, "->", winnow::ascii::space0), tir_core::Type::parse).parse_next(input)?;
                builder = builder.#ident(ty);
            };
        }
    }

    quote! {}
}

fn make_operand_parser(fields: &[OpFieldReceiver]) -> proc_macro2::TokenStream {
    let mut parsers = vec![];

    for f in fields {
        if let OpFieldAttrs::Operand = f.attrs {
            let operand_str = format!("{}", f.ident.as_ref().unwrap());
            let operand = f.ident.as_ref().unwrap();
            let ty = &f.ty;
            parsers.push(quote! {
                let (_, value) = winnow::combinator::separated_pair(
                    #operand_str,
                    (winnow::ascii::space0, "=", winnow::ascii::space0).recognize(),
                    #ty::parse
                ).parse_next(input)?;
                builder = builder.#operand(value);
            });
        }
    }

    quote! {
        #(#parsers)*
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
    let operand_parsers = make_operand_parser(&fields);
    let return_parser = make_return_type_parser(&fields);

    quote! {
      impl tir_core::OpAssembly for #op_name {
          fn print_assembly(&self, fmt: &mut dyn tir_core::IRFormatter) {
            #operand_printer

            fmt.write_direct("attrs = {");
            for (name, attr) in &self.r#impl.attrs {
                fmt.write_direct(name);
                fmt.write_direct(" = ");
                attr.print(fmt);
            }
            fmt.write_direct("}");

            #return_printer
          }

          fn parse_assembly(input: &mut tir_core::parser::ParseStream<'_>) -> tir_core::parser::AsmPResult<OpRef> {
            let mut builder = Self::builder(&input.state.get_context());
            #operand_parsers

            let attr_list = tir_core::parser::attr_list.parse_next(input)?;

            #return_parser

            let op = builder.build();
            op.borrow_mut().add_attrs(&attr_list);

            Ok(op)
          }
      }
    }
}
