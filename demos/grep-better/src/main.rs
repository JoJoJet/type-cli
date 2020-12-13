use type_cli::CLI;

#[derive(CLI)]
struct Grep(regex::Regex, #[optional] Option<String>);

fn main() -> Result<(), type_cli::Error> {
    let Grep(pattern, file) = Grep::process(std::env::args())?;
    if let Some(file) = file {
        eprintln!("Searching for `{}` in {}", pattern, file);
    } else {
        eprintln!("Searching for `{}` in stdin", pattern);
    }
    
    Ok(())
}
