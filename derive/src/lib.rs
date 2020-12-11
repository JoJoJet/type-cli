use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{self, Attribute, Fields, Ident, Item, Type};

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
            match parse_help(help) {
                Ok(help) => Some(help),
                Err(e) => return e.to_compile_error().into()
            }
        }
        else {
            None
        }
    }}
}

#[proc_macro_derive(CLI, attributes(help, named, flag, optional, variadic))]
pub fn cli(item: TokenStream) -> TokenStream {
    let cmd_ident;
    let parse_ty = crate_path!(Parse);
    let help_ty = crate_path!(HelpInfo);
    let err_ty = crate_path!(Error);
    let cli_ty = crate_path!(CLI);

    let input: Item = syn::parse(item).expect("failed to parse");

    let body = match input {
        Item::Enum(item) => {
            let mut _match = quote! {};

            let mut subc: Vec<String> = Vec::new();

            cmd_ident = item.ident;
            for syn::Variant {
                ident,
                attrs,
                fields,
                ..
            } in item.variants
            {
                let name = to_snake(&ident);

                let mut helpmsg = name.clone();
                if let Some(help) = try_help!(attrs.iter()) {
                    helpmsg.push('\t');
                    helpmsg.push_str(&help);
                }
                subc.push(helpmsg);

                let ctor = command(ident, attrs, fields);
                _match = quote! {
                    #_match
                    Some(#name) => {
                        #ctor
                    } ,
                };
            }

            let mut helpmsg = format!("Help - {}\n", to_snake(&cmd_ident));
            if let Some(help) = item.attrs.iter().find(|a| a.path.is_ident("help")) {
                match parse_help(help) {
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

                match ARGS_ITER.next().as_deref() {
                    #_match
                    Some("--help") | Some("-h") | None => return Ok(#parse_ty::Help(#help_ty(HELP))),
                    Some(sub) => return Err(#err_ty::UnknownSub(sub.to_string())),
                }
            }
        }
        Item::Struct(item) => {
            cmd_ident = item.ident.clone();
            let ctor = command(item.ident, item.attrs, item.fields);
            quote! {
                #ctor
            }
        }
        _ => panic!("Only allowed on structs and enums."),
    };

    let ret = quote! {
        impl #cli_ty for #cmd_ident {
            fn parse(mut ARGS_ITER : impl std::iter::Iterator<Item=String>) -> Result<#parse_ty<#cmd_ident>, #err_ty> {
                let _ = ARGS_ITER.next();
                let ret = {
                    #body
                };
                Ok(#parse_ty::Success(ret))
            }
        }
    };
    ret.into()
}

fn command(cmd_ident: Ident, attr: Vec<Attribute>, fields: Fields) -> TokenStream2 {
    let parse_ty = crate_path!(Parse);
    let help_ty = crate_path!(HelpInfo);
    let err_ty = crate_path!(Error);
    let arg_ty = crate_path!(Argument);
    let opt_ty = crate_path!(OptionalArg);

    let mut helpmsg = format!("Help - {}\n", to_snake(&cmd_ident));
    if let Some(help) = try_help!(attr.iter()) {
        helpmsg.push_str(&help);
        helpmsg.push_str("\n\n");
    }

    match fields {
        //
        // Named structs.
        Fields::Named(fields) => {
            struct Arg {
                ident: Ident,
                l_ident: Ident,
                arg_name: String,      // The cli-name of the argument. `--arg`
                short: Option<String>, // short name of the argument. `-a`
                name: String,          // The cli-name sans `--`
                help: Option<String>,
                ty: Type,
                required: bool,
                variadic: bool,
            }
            impl Arg {
                pub fn new(
                    ident: Ident,
                    short: Option<String>,
                    help: Option<String>,
                    ty: Type,
                    required: bool,
                    variadic: bool,
                ) -> Self {
                    let name = to_snake(&ident);
                    Self {
                        ident,
                        l_ident: format_ident!("{}", name),
                        arg_name: format!("--{}", name.replace("_", "-")),
                        name,
                        short: short.map(|s| format!("-{}", s)),
                        help,
                        ty,
                        required,
                        variadic,
                    }
                }
            }

            let short_reg = regex::Regex::new(r#"short\s*=\s*"(.*)""#).unwrap();

            let mut any_variadic = false;

            //
            // Process the arguments.
            let mut pos_args: Vec<Arg> = Vec::new();
            let mut named_args: Vec<Arg> = Vec::new();
            let mut flags: Vec<Arg> = Vec::new();
            for syn::Field {
                ident, attrs, ty, ..
            } in fields.named
            {
                let ident = ident.expect("field has an identifier");

                let required = !attrs.iter().any(|a| a.path.is_ident("optional"));
                let variadic = attrs.iter().any(|a| a.path.is_ident("variadic"));

                let help = try_help!(attrs.iter());

                // Named arguments.
                if let Some(named) = attrs.iter().find(|a| a.path.is_ident("named")) {
                    if variadic {
                        panic!("Named argument `{}` cannot be variadic.", ident.to_string());
                    }
                    let short = short_reg
                        .captures(&named.tokens.to_string())
                        .map(|cap| cap[1].to_string());
                    named_args.push(Arg::new(ident, short, help, ty, required, false));
                }
                // Flags.
                else if let Some(flag) = attrs.iter().find(|a| a.path.is_ident("flag")) {
                    if variadic {
                        panic!("Flag `{}` cannot be variadic.", ident.to_string());
                    }
                    let short = short_reg
                        .captures(&flag.tokens.to_string())
                        .map(|cap| cap[1].to_string());
                    flags.push(Arg::new(ident, short, help, ty, required, false));
                }
                // Positional arguments.
                else {
                    if required && pos_args.last().map_or(false, |a| !a.required) {
                        panic!("Required positional argument `{}` must come before any optional arguments.", ident.to_string());
                    }
                    if any_variadic {
                        panic!(
                            "Positional argument `{}` must come before the variadic argument.",
                            ident.to_string()
                        );
                    }
                    any_variadic = any_variadic || variadic;
                    pos_args.push(Arg::new(ident, None, help, ty, required, variadic));
                }
            }

            //
            // Generate help info for the arguments.
            if pos_args.len() + named_args.len() > 0 {
                helpmsg.push_str("ARGUMENTS:\n");
            }
            for arg in &pos_args {
                helpmsg.push_str("    ");
                helpmsg.push_str(&arg.name);
                if let Some(help) = &arg.help {
                    helpmsg.push('\t');
                    helpmsg.push_str(help);
                }
                if arg.variadic {
                    helpmsg.push('\t');
                    helpmsg.push_str("[variadic]");
                }
                if !arg.required {
                    helpmsg.push('\t');
                    helpmsg.push_str("[optional]");
                }
                helpmsg.push('\n');
            }
            for arg in &named_args {
                helpmsg.push_str("    ");
                if let Some(short) = &arg.short {
                    helpmsg.push_str(short);
                    helpmsg.push_str(", ");
                }
                helpmsg.push_str(&arg.arg_name);
                if let Some(help) = &arg.help {
                    helpmsg.push('\t');
                    helpmsg.push_str(help);
                }
                if !arg.required {
                    helpmsg.push('\t');
                    helpmsg.push_str("[optional]");
                }
                helpmsg.push('\n');
            }
            if pos_args.len() + named_args.len() > 0 {
                helpmsg.push('\n');
            }
            // Help info for flags.
            if flags.len() > 0 {
                helpmsg.push_str("FLAGS:\n");
            }
            for flag in &flags{
                helpmsg.push_str("    ");
                if let Some(short) = &flag.short {
                    helpmsg.push_str(short);
                    helpmsg.push_str(", ");
                }
                helpmsg.push_str(&flag.arg_name);
                if let Some(help) = &flag.help {
                    helpmsg.push('\t');
                    helpmsg.push_str(help);
                }
                helpmsg.push('\n');
            }

            //
            // Generate code to process the arguments at runtime.
            let mut declarations = quote! {
                let mut ARGS_ITER = ARGS_ITER.peekable();
            };
            // Code snippet to consume named arguments and flags.
            let consume_flags = {
                let mut match_args = quote! {};
                for Arg {
                    arg_name,
                    short,
                    l_ident,
                    ..
                } in &named_args
                {
                    declarations = quote! {
                        #declarations
                        let mut #l_ident: Option<String> = None;
                    };
                    let mut pattern = quote! { Some(#arg_name) };
                    if let Some(short) = short {
                        pattern = quote! { #pattern | Some(#short) };
                    }
                    match_args = quote! {
                        #match_args
                        #pattern => #l_ident = Some(ARGS_ITER.next().ok_or(#err_ty::ExpectedValue(#arg_name))?) ,
                    }
                }
                let mut match_flags = quote! {};
                let flag_ty = crate_path!(Flag);
                for Arg {
                    arg_name: flag,
                    short,
                    l_ident,
                    ty,
                    ..
                } in flags.iter()
                {
                    declarations = quote! {
                        #declarations
                        let mut #l_ident = <#ty>::default();
                    };
                    let mut pattern = quote! { Some(#flag) };
                    if let Some(short) = short {
                        pattern = quote! { #pattern | Some(#short) };
                    }
                    match_flags = quote! {
                        #match_flags
                        #pattern => #flag_ty::increment(&mut #l_ident) ,
                    };
                }

                let match_ = quote! {
                    match ARGS_ITER.next().as_deref() {
                        #match_args
                        #match_flags
                        Some("--help") | Some("-h") => return Ok(#parse_ty::Help(#help_ty(HELP))) ,
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
            let mut pos = quote! {};
            for (i, arg) in pos_args.iter().enumerate() {
                let &Arg {
                    ref l_ident,
                    required,
                    variadic,
                    ..
                } = arg;
                let i = i + 1;
                // Variadic arguments.
                if variadic {
                    declarations = quote! {
                        #declarations
                        let mut #l_ident = Vec::<String>::new();
                    };
                    pos = quote! {
                        #pos
                        while let Some(arg) = ARGS_ITER.next() {
                            #l_ident.push(arg);
                            #consume_flags
                        }
                    };
                }
                // Required arguments.
                else if required {
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
                // Optional arguments.
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
            let ctor = {
                let mut ctor = quote! {};
                for Arg {
                    ident,
                    l_ident,
                    required,
                    variadic,
                    ..
                } in pos_args
                {
                    // Collect args if variadic.
                    ctor = if variadic {
                        quote! {
                            #ctor
                            #ident : #l_ident.iter().map(#arg_ty::parse).collect::<Result<_, #err_ty>>()? ,
                        }
                    }
                    // Handle errors if required.
                    else if required {
                        quote! {
                            #ctor
                            #ident : #arg_ty::parse(#l_ident)? ,
                        }
                    }
                    // Allow defaults if optional.
                    else {
                        quote! {
                            #ctor
                            #ident: #opt_ty::map_parse(#l_ident)? ,
                        }
                    }
                }
                for Arg {
                    arg_name,
                    ident,
                    l_ident,
                    required,
                    ..
                } in named_args
                {
                    // Error handling if it's required.
                    ctor = if required {
                        quote! {
                            #ctor
                            #ident: #arg_ty::parse(#l_ident.ok_or(#err_ty::ExpectedNamed(#arg_name))?)? ,
                        }
                    }
                    // Defaults if it's optional
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

                quote! {
                    #cmd_ident { #ctor }
                }
            };

            quote! {
                const HELP: &str = #helpmsg;

                #declarations
                #consume_flags
                #pos
                // Return an error if there's an extra argument at the end.
                if let Some(a) = ARGS_ITER.next() {
                    return Err(#err_ty::ExtraArg(a));
                }
                #ctor
            }
        }

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
                let required = !attrs.iter().any(|a| a.path.is_ident("optional"));
                if required && args.last().map_or(false, |a| !a.required) {
                    panic!("Required argument at position `{}` must come before any optional arguments.", i+1);
                }
                let variadic = attrs.iter().any(|a| a.path.is_ident("variadic"));
                args.push(Arg { required, variadic });
            }

            //
            // Generate code to processs the arguments at runtime.
            let mut ctor = quote! {};
            for (i, Arg { required, variadic }) in args.into_iter().enumerate() {
                let i = i + 1;
                // Variadic arguments.
                ctor = if variadic {
                    // Run collect `by_ref` so it doesn't move the iterator.
                    quote! {
                        #ctor
                        ARGS_ITER.by_ref().map(#arg_ty::parse).collect::<Result<_, #err_ty>>()? ,
                    }
                }
                // Required arguments.
                else if required {
                    quote! {
                        #ctor
                        #arg_ty::parse(ARGS_ITER.next().ok_or(#err_ty::ExpectedPositional(#i))?)? ,
                    }
                }
                // Optional arguments.
                else {
                    quote! {
                        #ctor
                        #opt_ty::map_parse(ARGS_ITER.next())? ,
                    }
                }
            }
            quote! {
                let val = #cmd_ident (
                    #ctor
                );
                // Return an error if there's an extra argument at the end.
                if let Some(a) = ARGS_ITER.next() {
                    return Err(#err_ty::ExtraArg(a));
                }
                val
            }
        }
        Fields::Unit => todo!(),
    }
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
