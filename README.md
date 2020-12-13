# type-cli
type-cli is a convenient, strongly-typed command-line interface parser.

To start, let's create an interface for `grep`.

```rust
use type_cli::CLI;

#[derive(CLI)]
struct Grep(String, String);

fn main() {
    let Grep(pattern, file) = Grep::process();
    let pattern = regex::Regex::new(&pattern).unwrap();

    eprintln!("Searching for `{}` in {}", pattern, file);
}
```

Now, if we run the binary with arguments, they'll be properly parsed.
And if we miss an argument, it'll give a helpful error.

```
$ grep foo* myFile
Searching for `foo*` in myFile

$ grep foo*
Expected an argument at position `2`
```

However, this isn't exactly a faithful grep interface: in grep, the file is optional. Plus, that `unwrap()` is a little gross.

#

```rust
use type_cli::CLI;

#[derive(CLI)]
struct Grep(regex::Regex, #[optional] Option<String>);

fn main() {
    match Grep::process() {
        Grep(pattern, Some(file)) => eprintln!("Serching for `{}` in {}", pattern, file),
        Grep(pattern, None) => eprintln!("Searching for `{}` in stdin", pattern),
    }
}
```

What's that? We're accepting a `Regex` directly as an argument? In `type-cli`, any type that implements `FromStr` can be an argument.
Any parsing errors will be gracefully passed back to the user without you having to worry about it.

```
$ grep foo(
Error parsing positional argument `1`:
regex parse error:
    foo(
       ^
error: unclosed group
```

Here, you can also see that optional arguments must be annotated with `#[optional]`.

```
$ grep foo* myFile
Serching for `foo*` in myFile

$ grep foo*
Searching for `foo*` in stdin
```

This interface _still_ isn't faithful though; `grep` allows multiple files to be searched.

#

```rust
use type_cli::CLI;

#[derive(CLI)]
struct Grep(regex::Regex, #[variadic] Vec<String>);

fn main(){
    let Grep(pattern, file_list) = Grep::process();
    if file_list.is_empty() {
        eprintln!("Searching for `{}` in stdin", pattern);
    } else {
        eprint!("Searching for `{}` in ", pattern);
        file_list.iter().for_each(|f| eprint!("{}, ", f));
    }
}
```

If you annote the final field with `#[variadic]`, it will parse an arbitrary number of arguments.
This works for any collection that implements `FromIterator`.

```
$ grep foo*
Searching for `foo*` in stdin

$grep foo* myFile yourFile ourFile
Searching for `foo*` in myFile, yourFile, ourFile,
```

This still isn't ideal, though. None of the fields have names, and there's no flags or options!
Clearly, tuple structs are limiting us.

#

```rust
use type_cli::CLI;

#[derive(CLI)]
struct Grep {
    pattern: regex::Regex,

    #[named]
    file: String,

    #[flag(short = "i")]
    ignore_case: bool,
}

fn main() {
    let Grep { pattern, file, ignore_case } = Grep::process();
    eprint!("Searching for `{}` in {}", pattern, file);
    if ignore_case {
        eprint!(", ignoring case");
    }
    eprintln!();
}
```

Named arguments are annoted with `#[named]`, and that allows them to be passed to the command in any order.
By default, named arguments are still required, but they can also be marked with `#[optional]`.

```
$ grep foo*
Expected an argument named `--file`

$ grep foo* --file myFile
Searching for `foo*` in myFile
```

Flags are annoted with `#[flag]`, and are completely optional boolean or integer flags.
You can optionally specify a shorter form with `#[flag(short = "a")]` (this form also works for named arguments).

```
$ grep foo* --file myFile --ignore-case
Searching for `foo*` in myFile, ignoring case

$ grep foo* --file myFile -i
Searching for `foo*` in myFile, ignoring case
```

This seems well and good, but what if I want multiple commands in my application?

#

```rust
use type_cli::CLI;

#[derive(CLI)]
enum Cargo {
    New(String),
    Build {
        #[named] #[optional]
        target: Option<String>,
        #[flag]
        release: bool,
    },
    Clippy {
        #[flag]
        pedantic: bool,
    }
}

fn main() {
    match Cargo::process() {
        Cargo::New(name) => eprintln!("Creating new crate `{}`", name),
        Cargo::Build { target, release } => {
            let target = target.as_deref().unwrap_or("windows");
            if release {
                eprintln!("Building for {} in release", target);
            } else {
                eprintln!("Building for {}", target);
            }
        }
        Cargo::Clippy { pedantic: true } => eprintln!("Annoyingly checking your code."),
        Cargo::Clippy { pedantic: false } => eprintln!("Checking your code."),
    }
}
```

If you derive `CLI` on an enum, each variant will represent a subcommand.
Each subcommand is parsed with the same syntax as before.

Rust's pascal case will be automatically converted to the standard for shells:
`SubCommand` -> `sub-command`

```
$ cargo new myCrate
Creating new crate `myCrate`

$ cargo build
Building for windows

$ cargo build --target linux
Building for linux

$ cargo build --target linux --release
Building for linux in release

$ cargo clippy
Checking your code.

$ cargo clippy --pedantic
Annoyingly checking your code.
```

What about documentation?

#

```rust
use type_cli::CLI;

#[derive(CLI)]
#[help = "Build manager tool for rust"]
enum Cargo {
    New(String),

    #[help = "Build the current crate."]
    Build {
        #[named] #[optional]
        #[help = "the target platform"]
        target: Option<String>,

        #[flag]
        #[help = "build for release mode"]
        release: bool,
    },

    #[help = "Lint your code"]
    Clippy {
        #[flag]
        #[help = "include annoying and subjective lints"]
        pedantic: bool,
    }
}
```

`type-cli` will automatically generate a help screen for your commands.
If you annote a subcommand or argument with `#[help = ""]`, it will include your short description.
When shown, it will be sent to stderr and the process will exit with a nonzero status.

```
$ cargo
Help - cargo
Build manager tool for rust

SUBCOMMANDS:
    new
    build       Build the current crate.
    clippy      Lint your code
```

For enums, this will be shown if the command is called without specifying a subcommand.

```
$ cargo build --help
Help - build
Build the current crate.

ARGUMENTS:
    --target    the target platform     [optional]

FLAGS:
    --release   build for release mode


$ cargo clippy -h
Help - clippy
Lint your code

FLAGS:
    --pedantic  include annoying and subjective lints
```

For structs or subcommands, this will be called if the flag `--help` or `-h` is passed.
Help messages are not currently supported for tuple structs.