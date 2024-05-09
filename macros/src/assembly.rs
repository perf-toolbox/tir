use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn make_generic_ir_printer_parser(op_name: Ident) -> TokenStream {
    quote! {
      impl tir_core::OpAssembly for #op_name {
          fn print_assembly(&self, fmt: &mut dyn tir_core::IRFormatter) {
            todo!();
          }

          fn parse_assembly(input: &mut tir_core::parser::ParseStream<'_>) -> tir_core::parser::PResult<OpRef> {
            todo!();
          }
      }
    }
}
