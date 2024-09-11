pub mod errors {
    use std::fmt;
    use std::error::Error;

    #[derive(Debug)]
    pub struct DevCycleError {
        details: String
    }

    impl DevCycleError {
        pub fn new(msg: &str) -> DevCycleError {
            DevCycleError{details: msg.to_string()}
        }
    }

    impl fmt::Display for DevCycleError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.details)
        }
    }

    impl Error for DevCycleError {
        fn description(&self) -> &str {
            &self.details
        }
    }

}
pub const FAILED_TO_DECIDE_VARIATION: DevCycleError = DevCycleError{details: String::from("Failed to decide target variation") };
