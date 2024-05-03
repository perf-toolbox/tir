use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn make_generic_ir_printer_parser(op_name: Ident) -> TokenStream {
    quote! {
      impl IRAssembly for #op_name {
          fn print(&self, fmt: &mut dyn IRFormatter) {
            todo!();
          }

          fn parse<'s>(context: ContextRef, input: &mut &'s str) -> std::result::Result<Operation, ()> {
            Err(())
          }
      }
    }
}