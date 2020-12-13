use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{self, Attribute, Item};

macro_rules! crate_path {
    ($typ: tt) => {{
        let crate_name = proc_macro_crate::crate_name("type-cli")
            .expect("`type-cli` is present in `Cargo.toml`");
        let crate_name = quote::format_ident!("{}", crate_name);
        quote::quote! { ::#crate_name::$typ }
    }};
    () => {{
        let crate_name = proc_macro_crate::crate_name("type-cli")
            .expect("`type-cli` is present in `Cargo.toml`");
        let crate_name = quote::format_ident!("{}", crate_name);
        quote::quote! { ::#crate_name }
    }};
}

macro_rules! try_help {
    ($iter: expr) => {{
        let mut iter = $iter;
        if let Some(help) = iter.find(|a| a.path.is_ident("help")) {
            match $crate::parse_help(help) {
                Ok(help) => Some(help),
                Err(e) => return e.to_compile_error().into(),
            }
        } else {
            None
        }
    }};
}

mod enum_cmd;
mod struct_cmd;

#[proc_macro_derive(CLI, attributes(help, named, flag, optional, variadic))]
pub fn cli(item: TokenStream) -> TokenStream {
    let parse_ty = crate_path!(Parse);
    let err_ty = crate_path!(Error);
    let cli_ty = crate_path!(CLI);

    let input: Item = syn::parse(item).expect("failed to parse");

    let iter_ident = format_ident!("ARGS_ITER");
    let cmd_ident;

    let body = match input {
        Item::Enum(item) => {
            cmd_ident = item.ident;
            enum_cmd::parse(&cmd_ident, item.attrs, item.variants, &iter_ident)
        }
        Item::Struct(item) => {
            cmd_ident = item.ident.clone();
            struct_cmd::parse(item.ident, item.attrs, item.fields, &iter_ident)
        }
        _ => panic!("Only allowed on structs and enums."),
    };

    let ret = quote! {
        impl #cli_ty for #cmd_ident {
            fn parse(mut #iter_ident : impl std::iter::Iterator<Item=String>) -> Result<#parse_ty<#cmd_ident>, #err_ty> {
                let _ = #iter_ident.next();
                let ret = {
                    #body
                };
                Ok(#parse_ty::Success(ret))
            }
        }
    };
    ret.into()
}

fn parse_help(help: &Attribute) -> syn::Result<String> {
    match help.parse_meta()? {
        syn::Meta::NameValue(meta) => {
            if let syn::Lit::Str(help) = meta.lit {
                Ok(help.value())
            } else {
                Err(syn::Error::new_spanned(
                    help.tokens.clone(),
                    "Help message must be a string literal",
                ))
            }
        }
        _ => Err(syn::Error::new_spanned(
            help.tokens.clone(),
            r#"Help must be formatted as #[help = "msg"]"#,
        )),
    }
}

fn to_snake(ident: &impl ToString) -> String {
    let ident = ident.to_string();
    let mut val = String::with_capacity(ident.len());
    for (i, ch) in ident.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                val.push('-');
            }
            val.push(ch.to_ascii_lowercase());
        } else {
            val.push(ch);
        }
    }
    val
}
