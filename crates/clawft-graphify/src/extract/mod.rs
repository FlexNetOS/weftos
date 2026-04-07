//! Extraction pipeline: file discovery, AST extraction, semantic extraction.
//!
//! This module is gated behind the `ast-extract` feature. Semantic and vision
//! extraction live in sibling modules gated behind their own features.

// Re-export sub-modules as they are implemented.
// Phase 1-3 modules (AST, detect, lang, cross_file) will live here.
// For now we just declare the module structure.
