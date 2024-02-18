/// Represents a response from invoked command.
/// All fields are private to avoid situation, where
/// created object will be modified - result should be
/// constant.
#[derive(Debug)]
pub struct CrustResult {
    stdout: String,
    stderr: String,
    retcode: i32,
}

impl CrustResult {
    pub fn new(stdout: &str, stderr: &str, retcode: i32) -> Self {
        CrustResult {
            stdout: String::from(stdout),
            stderr: String::from(stderr),
            retcode,
        }
    }

    /// Getter for the possible command output.
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    /// Getter for the possible command error.
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    /// Getter for the return code.
    pub fn retcode(&self) -> i32 {
        self.retcode
    }

    /// Checks whether command has been completed successfuly.
    pub fn is_success(&self) -> bool {
        self.retcode == 0
    }
}

impl std::fmt::Display for CrustResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for CrustResult {
    /// Default value as successful invoke wihtout any stdout & stderr
    fn default() -> Self {
        CrustResult {
            stdout: String::from(""),
            stderr: String::from(""),
            retcode: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CrustResult;

    #[test]
    fn create_cmd_result() {
        let result = CrustResult::new("stdout", "stderr", 2);

        assert_eq!(result.retcode, 2);
        assert_eq!(result.stdout, "stdout");
        assert_eq!(result.stderr, "stderr");

        assert_eq!(result.retcode(), 2);
        assert_eq!(result.stdout(), "stdout");
        assert_eq!(result.stderr(), "stderr");

        assert!(!result.is_success());
    }

    #[test]
    fn create_cmd_result_default() {
        let result = CrustResult::default();

        assert_eq!(result.retcode, 0);
        assert_eq!(result.stdout, "");
        assert_eq!(result.stderr, "");
    }
}
