//! Extraction pipeline: file discovery, AST extraction, semantic extraction.
//!
//! The `detect` sub-module is always available (filesystem scanning only).
//! AST extraction (`ast`) and cross-file analysis (`cross_file`) are gated
//! behind the `ast-extract` feature.

pub mod detect;

#[cfg(feature = "ast-extract")]
pub mod ast;

#[cfg(feature = "ast-extract")]
pub mod cross_file;

#[cfg(feature = "ast-extract")]
pub mod lang;
