//! Helpers for turning stored resource paths into download URLs.
//!
//! The imported service never exposed an upload API, so object-storage
//! clients do not belong in the local-first application.

pub mod get_objects;
