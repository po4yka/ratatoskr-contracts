//! Canonical Document IR exchanged by Ratatoskr extraction and knowledge services.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod document;

pub use crate::document::{
    Document, DocumentAddress, DocumentBlock, DocumentProvenance, DocumentValidationError,
    ExtractionStrategy, LanguageTag,
};
