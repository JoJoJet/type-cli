use type_cli::CLI;

#[derive(CLI)]
struct Grep(regex::Regex, #[optional] Option<String>);

fn main() {
    match Grep::process() {
        Grep(pattern, Some(file)) => eprintln!("Serching for `{}` in {}", pattern, file),
        Grep(pattern, None) => eprintln!("Searching for `{}` in stdin", pattern),
    }
}
