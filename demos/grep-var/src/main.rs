use type_cli::CLI;

#[derive(CLI)]
struct Grep(regex::Regex, #[variadic] Vec<String>);

fn main() -> Result<(), type_cli::Error> {
    let Grep(pattern, file_list) = Grep::process(std::env::args())?;
    if file_list.is_empty() {
        eprintln!("Searching for `{}` in stdin", pattern);
    } else {
        eprint!("Searching for `{}` in ", pattern);
        for file in file_list {
            eprint!("{}, ", file);
        }
    }
    
    Ok(())
}