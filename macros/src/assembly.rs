use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn make_generic_ir_printer_parser(op_name: Ident) -> TokenStream {
    quote! {
      impl tir_core::Assembly for #op_name {
          fn print(&self, fmt: &mut dyn tir_core::IRFormatter) {
            todo!();
          }

          fn parse<'s>(context: tir_core::ContextRef, input: &mut &'s str) -> std::result::Result<tir_core::OpRef, ()> {
            Err(())
          }
      }
    }
}
