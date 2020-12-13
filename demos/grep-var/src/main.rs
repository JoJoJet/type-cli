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