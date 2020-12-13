use type_cli::CLI;

#[derive(CLI)]
#[help = "Build manager tool for rust"]
enum Cargo {
    New(String),

    #[help = "Build the current crate."]
    Build {
        #[named] #[optional]
        #[help = "the target platform"]
        target: Option<String>,

        #[flag]
        #[help = "build for release mode"]
        release: bool,
    },

    #[help = "Lint your code"]
    Clippy {
        #[flag]
        #[help = "include annoying and subjective lints"]
        pedantic: bool,
    }
}

fn main() {
    match Cargo::process() {
        Cargo::New(name) => eprintln!("Creating new crate `{}`", name),
        Cargo::Build { target, release } => {
            let target = target.as_deref().unwrap_or("windows");
            if release {
                eprintln!("Building for {} in release", target);
            } else {
                eprintln!("Building for {}", target);
            }
        }
        Cargo::Clippy { pedantic: true } => eprintln!("Annoyingly checking your code."),
        Cargo::Clippy { pedantic: false } => eprintln!("Checking your code."),
    }
}
