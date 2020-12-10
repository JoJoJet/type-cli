use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    self,
    Item,
    Ident,
    Attribute,
    Fields,
    Type
};
use quote::{format_ident, quote};


macro_rules! crate_path {
    ($typ: tt) => {
        {
            let crate_name = proc_macro_crate::crate_name("type-cli").expect("`type-cli` is present in `Cargo.toml`");
            let crate_name = quote::format_ident!("{}", crate_name);
            quote::quote!{ ::#crate_name::$typ }
        }
    };
    () => {
        {
            let crate_name = proc_macro_crate::crate_name("type-cli").expect("`type-cli` is present in `Cargo.toml`");
            let crate_name = quote::format_ident!("{}", crate_name);
            quote::quote!{ ::#crate_name }
        }
    }
}

#[proc_macro_derive(CLI, attributes(named, flag, optional, variadic))]
pub fn cli(item: TokenStream) -> TokenStream {
    let cmd_ident;
    let err_ty = crate_path!(Error);

    let input: Item = syn::parse(item).expect("failed to parse");

    let body = match input {
        Item::Enum(item) => {
            let mut _match = quote!{};

            cmd_ident = item.ident;
            for syn::Variant { ident, attrs, fields, .. } in item.variants {
                let name = to_snake(&ident);
                let ctor = command(ident, attrs, fields);
                _match = quote!{
                    #_match
                    Some(#name) => {
                        #ctor
                    } ,
                };
            }

            quote! {
                use #cmd_ident::*;
                match ARGS_ITER.next().as_deref() {
                    #_match
                    _ => panic!("Expected a subcommand.")
                }
            }
        },
        Item::Struct(item) => {
            cmd_ident = item.ident.clone();
            let ctor = command(item.ident, item.attrs, item.fields);
            quote!{
                #ctor
            }
        },
        _ => panic!("Only allowed on structs and enums.")
    };

    let ret = quote! {
        impl #cmd_ident {
            pub fn parse_cli(mut ARGS_ITER : impl std::iter::Iterator<Item=String>) -> Result<#cmd_ident, #err_ty> {
                let _ = ARGS_ITER.next();
                let ret = {
                    #body
                };
                Ok(ret)
            }
        }
    };
    ret.into()
}


fn command(cmd_ident: Ident, _attr: Vec<Attribute>, fields: Fields) -> TokenStream2 {

    let err_ty = crate_path!(Error);
    let arg_ty = crate_path!(Argument);
    let opt_ty = crate_path!(OptionalArg);

    match fields {
        //
        // Named structs.
        Fields::Named(fields) => {

            struct Arg {
                ident: Ident,
                l_ident: Ident,
                name: String,
                ty: Type,
                required: bool,
            }
            impl Arg {
                pub fn new(ident: Ident, ty: Type, required: bool) -> Self {
                    let name = to_snake(&ident);
                    let l_ident = format_ident!("{}", name);
                    Self { ident, l_ident, name: name.replace("_", "-"), ty, required }
                }
            }


            //
            // Process the arguments.
            let mut pos_args: Vec<Arg> = Vec::new();
            let mut named_args: Vec<Arg> = Vec::new();
            let mut flags: Vec<Arg> = Vec::new();
            for syn::Field { ident, attrs, ty, .. } in fields.named {
                let ident = ident.expect("field has an identifier");

                let required = !attrs.iter().any(|a| a.path.is_ident("optional"));
                
                // Named arguments.
                if attrs.iter().any(|a| a.path.is_ident("named")) {
                    named_args.push(Arg::new(ident, ty, required));
                }
                // Flags.
                else if attrs.iter().any(|a| a.path.is_ident("flag")) {
                    flags.push(Arg::new(ident, ty, required));
                }
                // Positional arguments.
                else {
                    if required && pos_args.last().map_or(false, |a| !a.required) {
                        panic!("Required positional argument `{}` must come before any optional arguments.", ident.to_string());
                    }
                    pos_args.push(Arg::new(ident, ty, required));
                }
            }


            //
            // Generate code to process the arguments at runtime.
            let mut declarations = quote! {
                let mut ARGS_ITER = ARGS_ITER.peekable();
            };
            // Code snippet to consume named arguments and flags.
            let consume_flags = {
                let mut match_args = quote! {};
                for Arg { name, l_ident, .. } in &named_args {
                    declarations = quote! {
                        #declarations
                        let mut #l_ident: Option<String> = None;
                    };
                    let arg = format!("--{}", name);
                    match_args = quote! {
                        #match_args
                        Some(#arg) => #l_ident = Some(ARGS_ITER.next().ok_or(#err_ty::ExpectedValue(#arg))?) ,
                    }
                }
                let mut match_flags = quote!{};
                let flag_ty = crate_path!(Flag);
                for Arg { name, l_ident, ty, .. } in flags.iter() {
                    declarations = quote! {
                        #declarations
                        let mut #l_ident = <#ty>::default();
                    };
                    let flag = format!("--{}", name);
                    match_flags = quote! {
                        #match_flags
                        Some(#flag) => #flag_ty::increment(&mut #l_ident) ,
                    };
                }

                let match_ = quote! {
                    match ARGS_ITER.next().as_deref() {
                        #match_args
                        #match_flags
                        Some(fl) => return Err(#err_ty::UnknownFlag(fl.to_string())),
                        _ => panic!("This shouldn't happen."),
                    }
                };
                quote! {
                    while ARGS_ITER.peek().map_or(false, |a| a.starts_with('-')) {
                        #match_
                    }
                }
            };
            // Code to consume positional arguments.
            let mut pos = quote!{};
            for (i, Arg { l_ident, required, ..}) in pos_args.iter().enumerate() {
                let i = i+1;
                if *required {
                    declarations = quote! {
                        #declarations
                        let #l_ident : String;
                    };
                    pos = quote! {
                        #pos
                        #l_ident = ARGS_ITER.next().ok_or(#err_ty::ExpectedPositional(#i))?;
                        #consume_flags
                    };
                }
                else {
                    declarations = quote! {
                        #declarations
                        let mut #l_ident: Option<String> = None;
                    };
                    pos = quote! {
                        #pos
                        if let Some(next) = ARGS_ITER.next() {
                            #l_ident = Some(next);
                            #consume_flags
                        }
                    };
                }
            }
            // Code to put the arguments in the constructor.
            let ctor =  {
                let mut ctor = quote!{};
                for Arg { ident, l_ident, required, .. } in pos_args {
                    ctor = if required { 
                        quote! {
                            #ctor
                            #ident : #arg_ty::parse(#l_ident)? ,
                        }
                    }
                    else {
                        quote! {
                            #ctor
                            #ident: #opt_ty::map_parse(#l_ident)? ,
                        }
                    }
                }
                for Arg { name, ident, l_ident, required, .. } in named_args {
                    ctor = if required {
                        quote! {
                            #ctor
                            #ident: #arg_ty::parse(#l_ident.ok_or(#err_ty::ExpectedNamed(#name))?)? ,
                        }
                    }
                    else {
                        quote! {
                            #ctor
                            #ident: #opt_ty::map_parse(#l_ident)? ,
                        }
                    }
                }
                for Arg { ident, l_ident, .. } in flags {
                    ctor = quote! {
                        #ctor
                        #ident: #l_ident ,
                    }
                }

                quote!{
                    #cmd_ident { #ctor } 
                }
            };
            
            
            quote! {
                {
                    #declarations
                    #consume_flags
                    #pos
                    #ctor
                }
            }
        },

        //
        // Tuple structs.
        Fields::Unnamed(fields) => {

            struct Arg {
                required: bool,
                variadic: bool,
            }

            //
            // Process the command's arguments.
            let mut args: Vec<Arg> = Vec::new();
            for (i, syn::Field { attrs, .. }) in fields.unnamed.into_iter().enumerate() {
                if args.last().map_or(false, |a| a.variadic) {
                    panic!("Variadic arguments must come last.");
                }
                let required = ! attrs.iter().any(|a| a.path.is_ident("optional"));
                if required && args.last().map_or(false, |a| !a.required) {
                    panic!("Required argument at position `{}` must come before any optional arguments.", i+1);
                }
                let variadic = attrs.iter().any(|a| a.path.is_ident("variadic"));
                args.push(Arg { required, variadic });
            }

            //
            // Generate code to processs the arguments at runtime.
            let mut ctor = quote!{};
            for (i, Arg { required, variadic }) in args.into_iter().enumerate() {
                let i = i+1;
                ctor = if variadic {
                    quote! {
                        #ctor
                        ARGS_ITER.map(#arg_ty::parse).collect::<Result<_, #err_ty>>()? ,
                    }
                }
                else if required {
                    quote! {
                        #ctor
                        #arg_ty::parse(ARGS_ITER.next().ok_or(#err_ty::ExpectedPositional(#i))?)? ,
                    }
                }
                else {
                    quote! {
                        #ctor
                        #opt_ty::map_parse(ARGS_ITER.next())? ,
                    }
                }
            }
            quote! {
                #cmd_ident (
                    #ctor
                )
            }
        },
        Fields::Unit => todo!()
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
        }
        else {
            val.push(ch);
        }
    }
    val
}
