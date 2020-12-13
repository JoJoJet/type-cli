use crate::to_snake;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{self, Ident, Type};

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

pub(super) struct Parser {
    cmd_ident: Ident,
    pos_args: Vec<Arg>,
    named_args: Vec<Arg>,
    flags: Vec<Arg>,
}
impl Parser {
    ///
    /// Process the fields of the struct into a form relevant to CLI.
    pub fn collect_args(cmd_ident: Ident, fields: syn::FieldsNamed) -> syn::Result<Self> {
        let short_reg = regex::Regex::new(r#"short\s*=\s*"(.*)""#).unwrap();

        let mut pos_args: Vec<Arg> = Vec::new();
        let mut named_args: Vec<Arg> = Vec::new();
        let mut flags: Vec<Arg> = Vec::new();
        let mut any_variadic = false;
        for syn::Field {
            ident, attrs, ty, ..
        } in fields.named
        {
            let ident = ident.expect("field has an identifier");

            let required = !attrs.iter().any(|a| a.path.is_ident("optional"));
            let variadic = attrs.iter().any(|a| a.path.is_ident("variadic"));

            //let help = try_help!(attrs.iter());
            let help = attrs
                .iter()
                .find(|a| a.path.is_ident("help"))
                .map(crate::parse_help)
                .transpose()?;

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

        Ok(Self {
            cmd_ident,
            pos_args,
            named_args,
            flags,
        })
    }
    ///
    /// Build help info about this command's arguments.
    pub fn build_help(&self, helpmsg: &mut String) {
        // Help info for arguments.
        let args_empty = self.pos_args.is_empty() && self.named_args.is_empty();
        if !args_empty {
            helpmsg.push_str("ARGUMENTS:\n");
        }
        for arg in &self.pos_args {
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
        for arg in &self.named_args {
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
        if !args_empty {
            helpmsg.push('\n');
        }
        // Help info for flags.
        if !self.flags.is_empty() {
            helpmsg.push_str("FLAGS:\n");
        }
        for flag in &self.flags {
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
    }
    ///
    ///
    pub fn into_ctor(self, iter: &Ident, help_ident: &Ident) -> TokenStream2 {
        let arg_ty = crate_path!(Argument);
        let opt_ty = crate_path!(OptionalArg);
        let parse_ty = crate_path!(Parse);
        let help_ty = crate_path!(HelpInfo);
        let err_ty = crate_path!(Error);
        let argref_ty = crate_path!(ArgRef);

        let Self {
            cmd_ident,
            pos_args,
            named_args,
            flags,
        } = self;
        let mut declarations = quote! {
            let mut #iter = #iter.peekable();
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
                    #pattern => #l_ident = Some(#iter.next().ok_or(#err_ty::ExpectedValue(#arg_name))?) ,
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
                match #iter.next().as_deref() {
                    #match_args
                    #match_flags
                    Some("--help") | Some("-h") => return Ok(#parse_ty::Help(#help_ty(#help_ident))) ,
                    Some(fl) => return Err(#err_ty::UnknownFlag(fl.to_string())),
                    _ => panic!("This shouldn't happen."),
                }
            };
            quote! {
                while #iter.peek().map_or(false, |a| a.starts_with('-')) {
                    #match_
                }
            }
        };
        //
        // Display the help message if called with no arguments.
        // If all of the arguments are optional, don't do this.
        let help_on_blank = if pos_args.iter().any(|a| a.required && !a.variadic) {
            quote! {
                if #iter.peek().is_none() {
                    return Ok(#parse_ty::Help(#help_ty(#help_ident)));
                }
            }
        } else {
            quote! {}
        };
        //
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
                    while let Some(arg) = #iter.next() {
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
                    #l_ident = #iter.next().ok_or(#err_ty::ExpectedPositional(#i))?;
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
                    if let Some(next) = #iter.next() {
                        #l_ident = Some(next);
                        #consume_flags
                    }
                };
            }
        }

        // Code to put the arguments in the constructor.
        let ctor = {
            let mut ctor = quote! {};
            for (i, arg) in pos_args.into_iter().enumerate() {
                let Arg {
                    ident,
                    l_ident,
                    required,
                    variadic,
                    ..
                } = arg;
                let i = i + 1;
                // Collect args if variadic.
                ctor = if variadic {
                    quote! {
                        #ctor
                        #ident : #l_ident.iter()
                            .enumerate()
                            .map(|(i, val)| #arg_ty::parse(val, #argref_ty::Positional(#i + i)))
                            .collect::<Result<_, #err_ty>>()? ,
                    }
                }
                // Handle errors if required.
                else if required {
                    quote! {
                        #ctor
                        #ident : #arg_ty::parse(#l_ident, #argref_ty::Positional(#i))? ,
                    }
                }
                // Allow defaults if optional.
                else {
                    quote! {
                        #ctor
                        #ident: #opt_ty::map_parse(#l_ident, #argref_ty::Positional(#i))? ,
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
                let argref = quote! { #argref_ty::Named(#arg_name) };
                // Error handling if it's required.
                ctor = if required {
                    quote! {
                        #ctor
                        #ident: #arg_ty::parse(#l_ident.ok_or(#err_ty::ExpectedNamed(#arg_name))?, #argref)? ,
                    }
                }
                // Defaults if it's optional
                else {
                    quote! {
                        #ctor
                        #ident: #opt_ty::map_parse(#l_ident, #argref)? ,
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

        quote! {{
            #declarations
            #help_on_blank
            #consume_flags
            #pos
            let val = #ctor;
            // Return an error if there's an extra argument at the end.
            if let Some(a) = #iter.next() {
                return Err(#err_ty::ExtraArg(a));
            }
            val
        }}
    }
}
