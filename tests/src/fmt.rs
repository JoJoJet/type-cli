#[derive(PartialEq, Eq, Debug, type_cli::CLI)]
#[help = "Format a string with an arbitrary number of values"]
pub struct Format(String, #[variadic] Vec<String>);

#[derive(PartialEq, Eq, Debug, type_cli::CLI)]
#[help = "Print one or two strings"]
pub struct Print(String, #[optional] Option<String>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format() {
        assert_eq!(
            process!(Format, "fmt" "arg1" "arg2").unwrap(),
            Format(
                "fmt".to_string(),
                vec!["arg1".to_string(), "arg2".to_string()]
            )
        );
    }

    #[test]
    fn print() {
        assert_eq!(
            process!(Print, "foo" "bar").unwrap(),
            Print("foo".to_string(), Some("bar".to_string()))
        );
    }
    #[test]
    fn print_opt() {
        assert_eq!(
            process!(Print, "foo").unwrap(),
            Print("foo".to_string(), None)
        );
    }
    #[test]
    #[should_panic(expected = "Unexpected positional argument `extra-arg`")]
    fn print_err() {
        process!(Print, "foo" "bar" "extra-arg").unwrap();
    }
}
