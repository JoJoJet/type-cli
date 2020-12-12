use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{self, Ident};

struct Arg {
    required: bool,
    variadic: bool,
}

pub(super) struct Parser {
    cmd_ident: Ident,
    args: Vec<Arg>,
}
impl Parser {
    /// Process the fields of the tuple struct from `syn` into a form relevant to CLI.
    pub fn collect_args(cmd_ident: Ident, fields: syn::FieldsUnnamed) -> Self {
        let mut args: Vec<Arg> = Vec::new();
        for (i, syn::Field { attrs, .. }) in fields.unnamed.into_iter().enumerate() {
            if args.last().map_or(false, |a| a.variadic) {
                panic!("Variadic arguments must come last.");
            }
            let required = !attrs.iter().any(|a| a.path.is_ident("optional"));
            if required && args.last().map_or(false, |a| !a.required) {
                panic!(
                    "Required argument at position `{}` must come before any optional arguments.",
                    i + 1
                );
            }
            let variadic = attrs.iter().any(|a| a.path.is_ident("variadic"));
            args.push(Arg { required, variadic });
        }

        Self { cmd_ident, args }
    }
    /// Convert this parser into ctor code for a CLI parser.
    pub fn into_ctor(self, iter: &Ident, _help_ident: &Ident) -> TokenStream2 {
        let arg_ty = crate_path!(Argument);
        let opt_ty = crate_path!(OptionalArg);
        let err_ty = crate_path!(Error);

        let Self { cmd_ident, args } = self;
        let mut ctor = quote! {};
        for (i, Arg { required, variadic }) in args.into_iter().enumerate() {
            let i = i + 1;
            // Variadic arguments.
            ctor = if variadic {
                // Run collect `by_ref` so it doesn't move the iterator.
                quote! {
                    #ctor
                    #iter.by_ref().map(#arg_ty::parse).collect::<Result<_, #err_ty>>()? ,
                }
            }
            // Required arguments.
            else if required {
                quote! {
                    #ctor
                    #arg_ty::parse(#iter.next().ok_or(#err_ty::ExpectedPositional(#i))?)? ,
                }
            }
            // Optional arguments.
            else {
                quote! {
                    #ctor
                    #opt_ty::map_parse(#iter.next())? ,
                }
            }
        }
        quote! {
            let val = #cmd_ident (
                #ctor
            );
            if let Some(a) = #iter.next() {
                return Err(#err_ty::ExtraArg(a));
            }
            val
        }
    }
}
