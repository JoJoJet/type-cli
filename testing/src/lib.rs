#[derive(PartialEq, Eq, Debug, type_cli::CLI)]
#[help = "Save or load files."]
pub enum FileSystem {
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
}

#[cfg(test)]
macro_rules! args {
    ($($st : literal)*) => {
        vec![$($st.to_string()),*].into_iter()
    }
}
#[cfg(test)]
macro_rules! parse {
    ($ty: ty, $($st: literal)*) => {
        <$ty as type_cli::CLI>::parse(args!("type-cli" $($st)*))
    }
}
#[cfg(test)]
macro_rules! process {
    ($ty: ty, $($st: literal)*) => {
        match parse!($ty, $($st)*) {
            Ok(type_cli::Parse::Success(val)) => Ok(val),
            Ok(type_cli::Parse::Help(h)) => panic!("{}", h),
            Err(e) => Err(e),
        }
    }
}

mod fmt;
mod misc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Help - file-system")]
    fn help() {
        process!(FileSystem,).unwrap();
    }
    #[test]
    #[should_panic(expected = "Help - file-system")]
    fn help2() {
        process!(FileSystem, "--help").unwrap();
    }
    #[test]
    #[should_panic(expected = "Help - file-system")]
    fn help3() {
        process!(FileSystem, "-h").unwrap();
    }

    #[test]
    fn save() {
        assert_eq!(
            process!(FileSystem, "save" "foo").unwrap(),
            FileSystem::Save {
                name: "foo".to_string(),
                verbose: false
            }
        );
    }
    #[test]
    fn save_verbose() {
        assert_eq!(
            process!(FileSystem, "save" "--verbose" "foo").unwrap(),
            FileSystem::Save {
                name: "foo".to_string(),
                verbose: true
            }
        );
    }
    #[test]
    fn save_v() {
        assert_eq!(
            process!(FileSystem, "save" "foo" "-v").unwrap(),
            FileSystem::Save {
                name: "foo".to_string(),
                verbose: true
            }
        );
    }
    #[test]
    #[should_panic(expected = "Expected an argument at position `1`")]
    fn save_err() {
        process!(FileSystem, "save" "-v").unwrap();
    }
    #[test]
    #[should_panic(expected = "Unexpected positional argument `too-many`")]
    fn save_err2() {
        process!(FileSystem, "save" "foo" "too-many").unwrap();
    }
    #[test]
    #[should_panic(expected = "Help - save")]
    fn save_help() {
        process!(FileSystem, "save" "--help").unwrap();
    }
    #[test]
    #[should_panic(expected = "Help - save")]
    fn save_help2() {
        process!(FileSystem, "save").unwrap();
    }

    #[test]
    fn load_file() {
        assert_eq!(
            process!(FileSystem, "load-file" "foo" "--time-out" "8").unwrap(),
            FileSystem::LoadFile {
                file: "foo".to_string(),
                bytes: Vec::new(),
                time_out: 8
            }
        );
    }
    #[test]
    fn load_file_bytes() {
        assert_eq!(
            process!(FileSystem, "load-file" "foo" "7" "255" "--time-out" "8").unwrap(),
            FileSystem::LoadFile {
                file: "foo".to_string(),
                bytes: vec![7, 255],
                time_out: 8
            }
        );
    }
    #[test]
    fn load_file_bytes2() {
        assert_eq!(
            process!(FileSystem, "load-file" "foo" "15" "48" "--time-out" "8" "29").unwrap(),
            FileSystem::LoadFile {
                file: "foo".to_string(),
                bytes: vec![15, 48, 29],
                time_out: 8
            }
        );
    }
    #[test]
    #[should_panic(expected = "Expected an argument named `--time-out`")]
    fn load_file_err() {
        process!(FileSystem, "load-file" "foo").unwrap();
    }
    #[test]
    #[should_panic(expected = "Expected a value after argument `--time-out`")]
    fn load_file_err2() {
        process!(FileSystem, "load-file" "foo" "--time-out").unwrap();
    }
    #[test]
    #[should_panic(expected = "Unknown flag `--lime-out`")]
    fn load_file_err3() {
        process!(FileSystem, "load-file" "foo" "--lime-out").unwrap();
    }
    #[test]
    #[should_panic(expected = "Help - load-file")]
    fn load_file_help() {
        process!(FileSystem, "load-file" "--help").unwrap();
    }
}
