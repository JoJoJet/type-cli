#[derive(PartialEq, Eq, Debug, type_cli::CLI)]
pub enum Command {
    Save {
        name: String,
        #[flag]
        verbose: bool,
    },
    LoadFile {
        file: String,
        #[named]
        time_out: u64,
    },
    Oof {
        ouch: String,
        #[named]
        #[short("c")]
        #[optional]
        count: Option<u32>,
    },
    Name {
        first: String,
        #[optional]
        last: Option<String>,
    },
    Print(String, #[optional] Option<String>),
    Format(String, #[variadic] Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! args {
        ($($st : literal)*) => {
            vec![$($st.to_string()),*].into_iter()
        }
    }

    macro_rules! parse {
        ($ty: ty, $($st: literal)*) => {
            <$ty>::parse_cli(args!("type-cli" $($st)*))
        }
    }

    #[test]
    fn save() {
        assert_eq!(
            parse!(Command, "save" "foo").unwrap(),
            Command::Save {
                name: "foo".to_string(),
                verbose: false
            }
        );
    }
    #[test]
    fn save_verbose() {
        assert_eq!(
            parse!(Command, "save" "--verbose" "foo").unwrap(),
            Command::Save {
                name: "foo".to_string(),
                verbose: true
            }
        );
    }
    #[test]
    #[should_panic(expected = "Expected an argument at position `1`")]
    fn save_err() {
        parse!(Command, "save").unwrap();
    }
    #[test]
    #[should_panic(expected = "Unexpected positional argument `too-many`")]
    fn save_err2() {
        parse!(Command, "save" "foo" "too-many").unwrap();
    }

    #[test]
    fn load_file() {
        assert_eq!(
            parse!(Command, "load-file" "foo" "--time-out" "8").unwrap(),
            Command::LoadFile {
                file: "foo".to_string(),
                time_out: 8
            }
        );
    }
    #[test]
    #[should_panic(expected = "Expected an argument named `time-out`")]
    fn load_file_err() {
        parse!(Command, "load-file" "foo").unwrap();
    }
    #[test]
    #[should_panic(expected = "Expected a value after argument `--time-out`")]
    fn load_file_err2() {
        parse!(Command, "load-file" "foo" "--time-out").unwrap();
    }
    #[test]
    #[should_panic(expected = "Unknown flag `--lime-out`")]
    fn load_file_err3() {
        parse!(Command, "load-file" "foo" "--lime-out").unwrap();
    }

    #[test]
    fn oof() {
        assert_eq!(
            parse!(Command, "oof" "foo" "--count" "4").unwrap(),
            Command::Oof {
                ouch: "foo".to_string(),
                count: Some(4)
            }
        );
        assert_eq!(
            parse!(Command, "oof" "foo").unwrap(),
            Command::Oof {
                ouch: "foo".to_string(),
                count: None
            }
        );
    }
    #[test]
    fn oof_short() {
        assert_eq!(
            parse!(Command, "oof" "-c" "12" "foo").unwrap(),
            Command::Oof {
                ouch: "foo".to_string(),
                count: Some(12)
            }
        );
    }
    #[test]
    #[should_panic(expected = "Error parsing string `kevin`")]
    fn oof_err() {
        parse!(Command, "oof" "foo" "--count" "kevin").unwrap();
    }

    #[test]
    fn name() {
        assert_eq!(
            parse!(Command, "name" "Robb" "Stark").unwrap(),
            Command::Name {
                first: "Robb".to_string(),
                last: Some("Stark".to_string())
            }
        );
        assert_eq!(
            parse!(Command, "name" "Pate").unwrap(),
            Command::Name {
                first: "Pate".to_string(),
                last: None
            }
        );
    }

    #[test]
    fn print() {
        assert_eq!(
            parse!(Command, "print" "foo" "bar").unwrap(),
            Command::Print("foo".to_string(), Some("bar".to_string()))
        );
    }
    #[test]
    fn print_opt() {
        assert_eq!(
            parse!(Command, "print" "foo").unwrap(),
            Command::Print("foo".to_string(), None)
        );
    }
    #[test]
    #[should_panic(expected = "Unexpected positional argument `extra-arg`")]
    fn print_err() {
        parse!(Command, "print" "foo" "bar" "extra-arg").unwrap();
    }

    #[test]
    fn format() {
        assert_eq!(
            parse!(Command, "format" "fmt" "arg1" "arg2").unwrap(),
            Command::Format(
                "fmt".to_string(),
                vec!["arg1".to_string(), "arg2".to_string()]
            )
        );
    }
}
