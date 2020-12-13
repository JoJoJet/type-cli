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
