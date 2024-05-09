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
          }

          fn parse_assembly(input: &mut tir_core::parser::ParseStream<'_>) -> tir_core::parser::PResult<OpRef> {
            todo!();
          }
      }
    }
}
