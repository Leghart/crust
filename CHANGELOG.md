# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- Podman testing image with corresponding script `manage_podman.py` to run test containers.
- Simple readme documenting podman script usage.
- Simple Gitlab CI with default testing jobs.
- Simple machines interface (tSCP, exec commands).
- Separated parsers with validation layer for every method-module
- Multithreaded copy between local and remote
- Build stage without DLL (muslrust) 

### Removed
- regex crate (replaced with manual checks)
- tSCP - rework for safe threads is required

[unreleased]: https://gitlab.com/Leghart/crust/-/tree/master
