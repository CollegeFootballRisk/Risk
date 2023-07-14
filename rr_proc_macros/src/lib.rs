use proc_macro::TokenStream;
use quote::{quote};
use syn::{parse_macro_input,Token, Ident, ItemFn};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use std::collections::HashSet as Set;

struct ArgsHoldingIdents {
  idents: Set<Ident>,
}

impl Parse for ArgsHoldingIdents {
  fn parse(args: ParseStream) -> Result<Self> {
    let vars = Punctuated::<Ident, Token![,]>::parse_terminated(args)?;
    Ok(ArgsHoldingIdents {
      idents: vars.into_iter().collect(),
    })
  }
}

/// `protect` macro
/// 
/// Macro takes optional OR arguments (v[0], v[1]...)
/// Use protect_and macro for dialing optional AND arguments (v[0], v[1], ...) instead
///
/// # Errors
/// - Returns crate::Error::Unauthorized if `User` not logged in at all
/// - Returns crate::Error::Unauthorized if `User` credentials are invalid
/// - Returns crate::Error::Forbidden if the `User`'s credentials are okay, but user is not
/// authorized to execute that transaction
///
/// # Panics
/// - Likely a database issue
#[proc_macro_attribute]
pub fn protect(attrs: TokenStream, stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as ItemFn);
    let args = parse_macro_input!(attrs as ArgsHoldingIdents);
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
        ..
    } = input;
    let stmts = &block.stmts;

    let permissions = args
    .idents
    .iter()
    .map(|ident| ident.to_string())
    .collect::<Vec<_>>()
    .join(", ");

    quote! {
        #(#attrs, _sconn: DbConn)* #vis #sig {
            // Check that the user provided a cookie
            let cookie = cookies
               .get_private("jwt")
               .ok_or(crate::Error::Unauthorized {})?;
           // Check to see that the cookie is valid
           let user = crate::model::User::from_cookie(&cookie)?;
           // Check to see if the user contains the requisite permission
           user.check_permissions(#permissions)?;
            #(#stmts)*
        }
    }.into()
}
