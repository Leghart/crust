use text_colorizer::Colorize;

use crate::interfaces::response::CrustResult;

/// Handles all possible errors from application.
/// If there was an error, exit app with code from error.
/// Otherwise return exit app with crust result retcode.
pub fn handle_result<EH: ExitHandler>(result: Result<CrustResult, CrustError>) {
    match result {
        Err(e) => EH::error(e),
        Ok(t) => EH::success(t),
    }
}

/// Handler for exit operation.
/// Must be a trait structure, to be able mocked
/// in tests (otherwise it will be always exited from tests)
pub trait ExitHandler {
    fn error(err: CrustError) -> !;
    fn success(result: CrustResult) -> !;
}

pub struct DefaultExitHandler {}

impl ExitHandler for DefaultExitHandler {
    fn error(err: CrustError) -> ! {
        eprintln!("{err}");
        std::process::exit(err.code.to_int());
    }

    fn success(result: CrustResult) -> ! {
        if result.is_success() {
            println!("{}", result.stdout().green());
        } else {
            println!("{}", result.stderr().red());
        }
        std::process::exit(result.retcode());
    }
}

/// Describes possible errors in app.
#[derive(Debug, Clone, PartialEq)]
pub enum ExitCode {
    Remote = 1,
    Local = 2,
    Std = 3,
    Ssh = 4,
    Internal = 5,
    Parser = 6,
}

/// Methods for enum
impl ExitCode {
    /// Transforms a enum value into integer.
    pub fn to_int(&self) -> i32 {
        self.clone() as i32
    }
}

/// Custom error struct to presentes every error from
/// `Result` which could be occured in application. Wrapper for
/// extern crates like ssh2 or standard libs.
#[derive(Debug, Clone)]
pub struct CrustError {
    pub code: ExitCode,
    pub message: String,
}

/// Display detailed error information along with information
/// about its source.
impl std::fmt::Display for CrustError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let err_msg = match self.code {
            ExitCode::Remote => format!("{}: {}", "[RemoteMachine]".red(), self.message),
            ExitCode::Local => format!("{}: {}", "[LocalMachine]".red(), self.message),
            ExitCode::Std => format!("{}: {}", "[StdError]".red(), self.message),
            ExitCode::Ssh => format!("{}: {}", "[SSH]".red(), self.message),
            ExitCode::Internal => format!("{}: {}", "[Internal]".red(), self.message),
            ExitCode::Parser => format!("{}: {}", "[Parser]".red(), self.message),
        };

        write!(f, "{}", err_msg)
    }
}

/// Convert the boxed error to CrustError
impl From<Box<dyn std::error::Error>> for CrustError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        CrustError {
            code: ExitCode::Internal,
            message: error.to_string(),
        }
    }
}

/// Handler for ssh2 standard error.
impl From<ssh2::Error> for CrustError {
    fn from(error: ssh2::Error) -> Self {
        CrustError {
            code: ExitCode::Ssh,
            message: error.to_string(),
        }
    }
}

/// Handler for std::io standard error.
impl From<std::io::Error> for CrustError {
    fn from(error: std::io::Error) -> Self {
        CrustError {
            code: ExitCode::Std,
            message: error.to_string(),
        }
    }
}

/// Handler for std::string standard error.
impl From<std::string::FromUtf8Error> for CrustError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        CrustError {
            code: ExitCode::Internal,
            message: error.to_string(),
        }
    }
}

/// Handler for logger standard error.
impl From<log::SetLoggerError> for CrustError {
    fn from(error: log::SetLoggerError) -> Self {
        CrustError {
            code: ExitCode::Parser,
            message: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct MockExitHandler;

    /// Changes process exit to panic with stderr message.
    impl ExitHandler for MockExitHandler {
        fn error(err: CrustError) -> ! {
            panic!("{err}");
        }

        fn success(result: CrustResult) -> ! {
            panic!("{}", result.retcode());
        }
    }

    #[cfg(not(feature = "CI"))]
    #[test]
    fn test_converts_box_error_into_crust_error() {
        #[derive(Debug)]
        struct CustomError;

        impl std::error::Error for CustomError {}
        impl std::fmt::Display for CustomError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Custom error")
            }
        }

        let custom_error: Box<dyn std::error::Error> = Box::new(CustomError);
        let crust_error: CrustError = custom_error.into();

        assert_eq!(
            format!("{crust_error}"),
            "\u{1b}[31m[Internal]\u{1b}[0m: Custom error".to_string()
        );
    }

    #[cfg(feature = "CI")]
    #[test]
    fn test_converts_box_error_into_crust_error_ci() {
        #[derive(Debug)]
        struct CustomError;

        impl std::error::Error for CustomError {}
        impl std::fmt::Display for CustomError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Custom error")
            }
        }

        let custom_error: Box<dyn std::error::Error> = Box::new(CustomError);
        let crust_error: CrustError = custom_error.into();

        assert_eq!(
            format!("{crust_error}"),
            "[Internal]: Custom error".to_string()
        );
    }

    #[cfg(not(feature = "CI"))]
    #[test]
    fn test_converts_fromstring_error_into_crust_error() {
        let fromstr_error = String::from_utf8(vec![0xC3, 0x28]).err().unwrap();
        let crust_error: CrustError = fromstr_error.into();

        assert_eq!(
            format!("{crust_error}"),
            "\u{1b}[31m[Internal]\u{1b}[0m: invalid utf-8 sequence of 1 bytes from index 0"
                .to_string()
        );
    }

    #[cfg(feature = "CI")]
    #[test]
    fn test_converts_fromstring_error_into_crust_error_ci() {
        let fromstr_error = String::from_utf8(vec![0xC3, 0x28]).err().unwrap();
        let crust_error: CrustError = fromstr_error.into();

        assert_eq!(
            format!("{crust_error}"),
            "[Internal]: invalid utf-8 sequence of 1 bytes from index 0".to_string()
        );
    }

    #[cfg(not(feature = "CI"))]
    #[test]
    fn test_converts_stdio_error_into_crust_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "Custom IO error");
        let crust_error: CrustError = io_error.into();

        assert_eq!(
            format!("{crust_error}"),
            "\u{1b}[31m[StdError]\u{1b}[0m: Custom IO error".to_string()
        );
    }

    #[cfg(feature = "CI")]
    #[test]
    fn test_converts_stdio_error_into_crust_error_ci() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "Custom IO error");
        let crust_error: CrustError = io_error.into();

        assert_eq!(
            format!("{crust_error}"),
            "[StdError]: Custom IO error".to_string()
        );
    }

    #[cfg(not(feature = "CI"))]
    #[test]
    #[should_panic(expected = "\u{1b}[31m[Internal]\u{1b}[0m: test msg")]
    fn test_handle_result_error() {
        let err: Result<_, CrustError> = Err(CrustError {
            code: ExitCode::Internal,
            message: "test msg".to_string(),
        });

        handle_result::<MockExitHandler>(err);
    }

    #[cfg(feature = "CI")]
    #[test]
    #[should_panic(expected = "[Internal]: test msg")]
    fn test_handle_result_error_ci() {
        let err: Result<_, CrustError> = Err(CrustError {
            code: ExitCode::Internal,
            message: "test msg".to_string(),
        });

        handle_result::<MockExitHandler>(err);
    }

    #[test]
    #[should_panic(expected = "0")]
    fn test_handle_result_success() {
        let result = Ok(CrustResult::default());
        handle_result::<MockExitHandler>(result);
    }
}
