use std::iter::IntoIterator as IntoIter;

use super::to_snake;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{self, Attribute, Ident, Variant};

pub(super) fn parse(
    cmd_ident: &Ident,
    attrs: Vec<Attribute>,
    variants: impl IntoIter<Item = Variant>,
    iter_ident: &Ident,
) -> TokenStream2 {
    let parse_ty = crate_path!(Parse);
    let help_ty = crate_path!(HelpInfo);
    let err_ty = crate_path!(Error);

    let mut subc: Vec<String> = Vec::new();

    let mut _match = quote! {};

    for Variant {
        ident,
        attrs,
        fields,
        ..
    } in variants
    {
        let name = to_snake(&ident);

        let mut helpmsg = name.clone();
        if let Some(help) = try_help!(attrs.iter()) {
            helpmsg.push('\t');
            helpmsg.push_str(&help);
        }
        subc.push(helpmsg);

        let ctor = super::struct_cmd::parse(ident, attrs, fields, iter_ident);
        _match = quote! {
            #_match
            Some(#name) => {
                #ctor
            } ,
        };
    }

    let mut helpmsg = format!("Help - {}\n", to_snake(cmd_ident));
    if let Some(help) = attrs.iter().find(|a| a.path.is_ident("help")) {
        match super::parse_help(help) {
            Ok(help) => {
                helpmsg.push_str(&help);
                helpmsg.push_str("\n\n");
            }
            Err(e) => return e.to_compile_error().into(),
        }
    }

    helpmsg.push_str("SUBCOMMANDS:\n");
    for subc in subc {
        helpmsg.push_str("    ");
        helpmsg.push_str(&subc);
        helpmsg.push('\n');
    }

    quote! {
        use #cmd_ident::*;

        const HELP: &str = #helpmsg;

        match #iter_ident.next().as_deref() {
            #_match
            Some("--help") | Some("-h") | None => return Ok(#parse_ty::Help(#help_ty(HELP))),
            Some(sub) => return Err(#err_ty::UnknownSub(sub.to_string())),
        }
    }
}
