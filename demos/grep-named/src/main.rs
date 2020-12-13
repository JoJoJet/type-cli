use type_cli::CLI;

#[derive(CLI)]
struct Grep {
    pattern: regex::Regex,

    #[named]
    file: String,

    #[flag(short = "v")]
    invert: bool,
}

fn main() -> Result<(), type_cli::Error> {
    let Grep { pattern, file, invert } = Grep::process(std::env::args())?;
    if invert {
        eprintln!("Searching for anything that doesn't match `{}` in {}", pattern, file);
    } else {
        eprintln!("Searching for `{}` in {}", pattern, file);
    }

    Ok(())
}
