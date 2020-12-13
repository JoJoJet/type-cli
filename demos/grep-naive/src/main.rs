use type_cli::CLI;

#[derive(CLI)]
struct Grep(String, String);

fn main() {
    let Grep(pattern, file) = Grep::process();
    let pattern = regex::Regex::new(&pattern).unwrap();

    eprintln!("Searching for `{}` in {}", pattern, file);
}
