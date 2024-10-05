use proc_macro::TokenStream;
use quote::quote;
use syn::{braced, parse::Parse, parse_macro_input, punctuated::Punctuated, Token};

struct MatchArm {
    op: Option<syn::Ident>,
    body: syn::Expr,
}

impl Parse for MatchArm {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.parse::<Token![_]>().is_ok() {
            input.parse::<Token![=>]>()?;
            let body: syn::Expr = input.parse()?;

            return Ok(Self { op: None, body });
        }

        let op: syn::Ident = input.parse()?;
        let op = Some(op);
        input.parse::<Token![=>]>()?;
        let body: syn::Expr = input.parse()?;

        Ok(Self { op, body })
    }
}

struct MatchInput {
    target: syn::Ident,
    arms: Vec<MatchArm>,
    catch_all: Option<MatchArm>,
}

impl Parse for MatchInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let target: syn::Ident = input.parse()?;
        let body;
        braced!(body in input);
        let all_arms = Punctuated::<MatchArm, Token![,]>::parse_terminated(&body)?;
        let mut arms = vec![];
        let mut catch_all = None;

        for arm in all_arms {
            if arm.op.is_none() {
                catch_all = Some(arm);
            } else {
                arms.push(arm);
            }
        }

        Ok(Self {
            target,
            arms,
            catch_all,
        })
    }
}

/// Expand match_op! { ... } helper macro
pub(crate) fn op_matcher(input: TokenStream) -> TokenStream {
    let match_input = parse_macro_input!(input as MatchInput);

    let mut tokens = vec![];

    let mut it = match_input.arms.into_iter();

    let first = it.next().unwrap();
    let ty = first.op.unwrap();
    let body = first.body;

    let target = match_input.target;

    tokens.push(quote! {
      if (#target.borrow().type_id() == std::any::TypeId::of::<#ty>()) {
        let concrete = tir_core::utils::op_cast::<#ty>(#target).unwrap();
        let lambda = #body;
        lambda(concrete)
      }
    });

    for arm in it {
        let ty = arm.op.unwrap();
        let body = arm.body;
        tokens.push(quote! {
          else if (#target.borrow().type_id() == std::any::TypeId::of::<#ty>()) {
            let concrete = tir_core::utils::op_cast::<#ty>(#target).unwrap();
            let lambda = #body;
            lambda(concrete)
          }
        });
    }

    if let Some(catch_all) = match_input.catch_all {
        let body = catch_all.body;

        tokens.push(quote! {
          else {
            let lambda = #body;
            lambda()
          }
        });
    } else {
        tokens.push(quote! {
          else {
            unreachable!()
          }
        });
    }

    quote! {
      #(
        #tokens
      )*
    }
    .into()
}
