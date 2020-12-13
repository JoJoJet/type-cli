use type_cli::CLI;

#[derive(CLI)]
struct Grep(String, String);

fn main() -> Result<(), type_cli::Error> {
    let Grep(pattern, file) = Grep::process(std::env::args())?;
    let pattern = regex::Regex::new(&pattern).unwrap();

    eprintln!("Searching for `{}` in {}", pattern, file);
    
    Ok(())
}
