pub mod download;
pub mod parser;
pub mod upload;
pub mod utils;

use std::path::PathBuf;

use crate::error::CrustError;
use crate::machine::base::MachineType;

const BUF_SIZE: usize = 4096;

pub trait Tscp {
    /// Merges a chunks of source into destination on `dst` path.
    fn merge(&self, dst: &str) -> Result<(), CrustError>;

    /// Splits source data into chunks with passed size. Every chunk
    /// will be saved on temporaty directory, created per each structure.
    fn split(&mut self, size: u64, data: &str) -> Result<Vec<PathBuf>, CrustError>;

    /// Getter for machine (common interface provided by Machine trait).
    fn get_machine(&self) -> MachineType;

    /// Getter for string preoresentation of machine. Used in
    /// connection in ssh2 crate.
    fn get_address(&self) -> String;

    /// Creates a vector of chunks from a string where every chunk is
    /// in a new line.
    fn _string_chunks_to_vec(&self, binding: String) -> Result<Vec<PathBuf>, CrustError> {
        let result: Vec<String> = binding
            .split('\n')
            .collect::<Vec<&str>>()
            .iter()
            .filter(|&v| !v.is_empty())
            .map(|v| v.to_string())
            .collect();

        let vec_of_paths: Vec<PathBuf> = result.into_iter().map(PathBuf::from).collect();

        Ok(vec_of_paths)
    }
}
