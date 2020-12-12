#[derive(PartialEq, Eq, Debug, type_cli::CLI)]
#[help = "Format a person's name"]
pub struct Name {
    #[help = "First name"]
    first: String,
    #[optional]
    #[help = "Last name"]
    last: Option<String>,
}

#[derive(PartialEq, Eq, Debug, type_cli::CLI)]
pub struct Oof {
    ouch: String,
    #[named(short = "c")]
    #[optional]
    count: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name() {
        assert_eq!(
            process!(Name, "Robb" "Stark").unwrap(),
            Name {
                first: "Robb".to_string(),
                last: Some("Stark".to_string())
            }
        );
    }
    #[test]
    fn name_op() {
        assert_eq!(
            process!(Name, "Pate").unwrap(),
            Name {
                first: "Pate".to_string(),
                last: None
            }
        );
    }
    #[test]
    #[should_panic(expected = "Help - name")]
    fn name_help() {
        process!(Name, "--help").unwrap();
    }

    #[test]
    fn oof() {
        assert_eq!(
            process!(Oof, "foo" "--count" "4").unwrap(),
            Oof {
                ouch: "foo".to_string(),
                count: Some(4)
            }
        );
        assert_eq!(
            process!(Oof, "foo").unwrap(),
            Oof {
                ouch: "foo".to_string(),
                count: None
            }
        );
    }
    #[test]
    fn oof_short() {
        assert_eq!(
            process!(Oof, "-c" "12" "foo").unwrap(),
            Oof {
                ouch: "foo".to_string(),
                count: Some(12)
            }
        );
    }
    #[test]
    #[should_panic(expected = "Error parsing string `kevin`")]
    fn oof_err() {
        process!(Oof, "foo" "--count" "kevin").unwrap();
    }
}
