

pub mod custom_errors {
    use std;
    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    pub struct CustomRustixFrontendError {
        pub err: String,
    }

    impl Error for CustomRustixFrontendError {
        fn description(&self) -> &str {
            "Something bad happened"
        }
    }

    impl fmt::Display for CustomRustixFrontendError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Oh no, something bad went down")
        }
    }
}