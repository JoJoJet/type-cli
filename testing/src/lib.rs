#[derive(PartialEq, Eq, Debug, type_cli::CLI)]
#[help = "Does too many things"]
pub enum Command {
    #[help = "Save a file"]
    Save {
        #[help = "Name of the destination file"]
        name: String,
        #[flag(short = "v")]
        #[help = "Print on success"]
        verbose: bool,
    },
    #[help = "Load a file"]
    LoadFile {
        #[help = "The file to load"]
        file: String,
        #[variadic]
        #[help = "Some options or something idk"]
        bytes: Vec<u8>,
        #[named]
        #[help = "How long to wait before cancelling (ms)"]
        time_out: u64,
    },
    Oof {
        ouch: String,
        #[named(short = "c")]
        #[optional]
        count: Option<u32>,
    },
    #[help = "Format a person's name"]
    Name {
        #[help = "First name"]
        first: String,
        #[optional]
        #[help = "Last name"]
        last: Option<String>,
    },
    #[help = "Print one or two strings"]
    Print(String, #[optional] Option<String>),
    #[help = "Format a string with an arbitrary number of values"]
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
            <$ty as type_cli::CLI>::parse(args!("type-cli" $($st)*))
        }
    }

    macro_rules! process {
        ($ty: ty, $($st: literal)*) => {
            match parse!($ty, $($st)*) {
                Ok(type_cli::Parse::Success(val)) => Ok(val),
                Ok(type_cli::Parse::Help(h)) => panic!("{}", h),
                Err(e) => Err(e),
            }
        }
    }

    #[test]
    #[should_panic(expected = "Help - command")]
    fn help() {
        process!(Command,).unwrap();
    }
    #[test]
    #[should_panic(expected = "Help - command")]
    fn help2() {
        process!(Command, "--help").unwrap();
    }
    #[test]
    #[should_panic(expected = "Help - command")]
    fn help3() {
        process!(Command, "-h").unwrap();
    }

    #[test]
    fn save() {
        assert_eq!(
            process!(Command, "save" "foo").unwrap(),
            Command::Save {
                name: "foo".to_string(),
                verbose: false
            }
        );
    }
    #[test]
    fn save_verbose() {
        assert_eq!(
            process!(Command, "save" "--verbose" "foo").unwrap(),
            Command::Save {
                name: "foo".to_string(),
                verbose: true
            }
        );
    }
    #[test]
    fn save_v() {
        assert_eq!(
            process!(Command, "save" "foo" "-v").unwrap(),
            Command::Save {
                name: "foo".to_string(),
                verbose: true
            }
        );
    }
    #[test]
    #[should_panic(expected = "Expected an argument at position `1`")]
    fn save_err() {
        process!(Command, "save").unwrap();
    }
    #[test]
    #[should_panic(expected = "Unexpected positional argument `too-many`")]
    fn save_err2() {
        process!(Command, "save" "foo" "too-many").unwrap();
    }
    #[test]
    #[should_panic(expected = "Help - save")]
    fn save_help() {
        process!(Command, "save" "--help").unwrap();
    }

    #[test]
    fn load_file() {
        assert_eq!(
            process!(Command, "load-file" "foo" "--time-out" "8").unwrap(),
            Command::LoadFile {
                file: "foo".to_string(),
                bytes: Vec::new(),
                time_out: 8
            }
        );
    }
    #[test]
    fn load_file_bytes() {
        assert_eq!(
            process!(Command, "load-file" "foo" "7" "255" "--time-out" "8").unwrap(),
            Command::LoadFile {
                file: "foo".to_string(),
                bytes: vec![7, 255],
                time_out: 8
            }
        );
    }
    #[test]
    fn load_file_bytes2() {
        assert_eq!(
            process!(Command, "load-file" "foo" "15" "48" "--time-out" "8" "29").unwrap(),
            Command::LoadFile {
                file: "foo".to_string(),
                bytes: vec![15, 48, 29],
                time_out: 8
            }
        );
    }
    #[test]
    #[should_panic(expected = "Expected an argument named `--time-out`")]
    fn load_file_err() {
        process!(Command, "load-file" "foo").unwrap();
    }
    #[test]
    #[should_panic(expected = "Expected a value after argument `--time-out`")]
    fn load_file_err2() {
        process!(Command, "load-file" "foo" "--time-out").unwrap();
    }
    #[test]
    #[should_panic(expected = "Unknown flag `--lime-out`")]
    fn load_file_err3() {
        process!(Command, "load-file" "foo" "--lime-out").unwrap();
    }
    #[test]
    #[should_panic(expected = "Help - load-file")]
    fn load_file_help() {
        process!(Command, "load-file" "--help").unwrap();
    }

    #[test]
    fn oof() {
        assert_eq!(
            process!(Command, "oof" "foo" "--count" "4").unwrap(),
            Command::Oof {
                ouch: "foo".to_string(),
                count: Some(4)
            }
        );
        assert_eq!(
            process!(Command, "oof" "foo").unwrap(),
            Command::Oof {
                ouch: "foo".to_string(),
                count: None
            }
        );
    }
    #[test]
    fn oof_short() {
        assert_eq!(
            process!(Command, "oof" "-c" "12" "foo").unwrap(),
            Command::Oof {
                ouch: "foo".to_string(),
                count: Some(12)
            }
        );
    }
    #[test]
    #[should_panic(expected = "Error parsing string `kevin`")]
    fn oof_err() {
        process!(Command, "oof" "foo" "--count" "kevin").unwrap();
    }

    #[test]
    fn name() {
        assert_eq!(
            process!(Command, "name" "Robb" "Stark").unwrap(),
            Command::Name {
                first: "Robb".to_string(),
                last: Some("Stark".to_string())
            }
        );
        assert_eq!(
            process!(Command, "name" "Pate").unwrap(),
            Command::Name {
                first: "Pate".to_string(),
                last: None
            }
        );
    }
    #[test]
    #[should_panic(expected = "Help - name")]
    fn name_help() {
        process!(Command, "name" "--help").unwrap();
    }

    #[test]
    fn print() {
        assert_eq!(
            process!(Command, "print" "foo" "bar").unwrap(),
            Command::Print("foo".to_string(), Some("bar".to_string()))
        );
    }
    #[test]
    fn print_opt() {
        assert_eq!(
            process!(Command, "print" "foo").unwrap(),
            Command::Print("foo".to_string(), None)
        );
    }
    #[test]
    #[should_panic(expected = "Unexpected positional argument `extra-arg`")]
    fn print_err() {
        process!(Command, "print" "foo" "bar" "extra-arg").unwrap();
    }

    #[test]
    fn format() {
        assert_eq!(
            process!(Command, "format" "fmt" "arg1" "arg2").unwrap(),
            Command::Format(
                "fmt".to_string(),
                vec!["arg1".to_string(), "arg2".to_string()]
            )
        );
    }
}
