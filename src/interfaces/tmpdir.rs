use crate::error::CrustError;
use std::path::PathBuf;

/// Sets of methods required to handle temporary directory -
/// mainly used in scp-like methods to store a temp files.
pub trait TemporaryDirectory {
    /// Creates a temporaty dir on self machine.
    fn create_tmpdir(&mut self) -> Result<PathBuf, CrustError>;

    /// Removes temporary directory.
    fn remove_tmpdir(&self) -> Result<(), CrustError>;

    /// Gets absolute path to existing directory.
    fn get_tmpdir(&self) -> &PathBuf;

    /// Checks whether temporary directory was created.
    /// TODO: check if dir still exists (could be removed)
    fn tmpdir_exists(&self) -> bool;

    /// Determines whether the directory can be removed
    /// (required by multi-threaded approach to avoid case when
    /// one of thread removes tmp_dir with results collected from
    /// other threads).
    fn can_be_removed(&self) -> bool;

    /// Creates a file inside temporary directory with
    /// requested name.
    fn create_tmpdir_content(&self, filename: &str) -> Result<PathBuf, CrustError>;
}
