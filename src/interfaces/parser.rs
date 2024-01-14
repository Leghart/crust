use crate::error::CrustError;

/// Interface for custom parsers validation.
/// Allows to check complex arguments or
/// transformates some args to others.
pub trait Validation {
    fn validate(&mut self) -> Result<(), CrustError>;
}
