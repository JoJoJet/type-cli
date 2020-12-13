use super::to_snake;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{self, Attribute, Fields, Ident};

mod named;
mod tuple;

pub(super) fn parse(
    cmd_ident: Ident,
    attr: Vec<Attribute>,
    fields: Fields,
    iter_ident: &Ident,
) -> TokenStream2 {
    let mut helpmsg = format!("Help - {}\n", to_snake(&cmd_ident));
    if let Some(help) = try_help!(attr.iter()) {
        helpmsg.push_str(&help);
        helpmsg.push_str("\n\n");
    }

    let help_ident = format_ident!("HELP");

    let ctor = match fields {
        //
        // Named structs.
        Fields::Named(fields) => {
            let parser = match named::Parser::collect_args(cmd_ident, fields) {
                Ok(parser) => parser,
                Err(e) => return e.to_compile_error(),
            };
            parser.build_help(&mut helpmsg);
            parser.into_ctor(iter_ident, &help_ident)
        }

        //
        // Tuple structs.
        Fields::Unnamed(fields) => {
            let parser = tuple::Parser::collect_args(cmd_ident, fields);
            parser.into_ctor(iter_ident, &help_ident)
        }
        Fields::Unit => todo!(),
    };

    quote! {
        const #help_ident: &str = #helpmsg;
        #ctor
    }
}
