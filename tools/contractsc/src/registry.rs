//! The single enumeration point for every root type this repository publishes.
//!
//! Explicit and hand-written. No reflection, no build script, no `inventory`/`linkme`, no glob
//! over `.rs` files: every one of those makes the generated output depend on link order or
//! filesystem order, which is exactly what `ARCHITECTURE.md` S11 forbids.
//!
//! The dependency of this crate on the contract crates **is** the enumeration mechanism: a
//! contract that is not reachable from here cannot be generated, and `contracts.toml` rule R2
//! proves the two lists are the same set in both directions.

use std::collections::BTreeMap;

use ratatoskr_event_envelope::EventPayload;

/// One publishable root type.
pub struct RootType {
    /// Must match a `contract.root_type.rust_path` in `contracts.toml` byte for byte.
    ///
    /// **Authored**, not `stringify!`d, so the R2 cross-check compares two authored strings and
    /// cannot be defeated by token spacing.
    pub rust_path: &'static str,

    /// Builds the root schema for this type with the supplied generator.
    pub schema: fn(&mut schemars::SchemaGenerator) -> schemars::Schema,

    /// Deserialize then re-serialize; drives the fixture round-trip and lossless-parse checks.
    pub roundtrip: fn(&serde_json::Value) -> Result<serde_json::Value, String>,
}

impl core::fmt::Debug for RootType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RootType")
            .field("rust_path", &self.rust_path)
            .finish_non_exhaustive()
    }
}

impl RootType {
    /// The Rust type's short name, used as the schema `title` and as the lint's declaring-type
    /// key, e.g. `EventEnvelope` for `ratatoskr_event_envelope::EventEnvelope`.
    #[must_use]
    pub fn short_name(&self) -> &'static str {
        short_name(self.rust_path)
    }
}

/// The short name of a `::`-qualified Rust path.
#[must_use]
pub fn short_name(rust_path: &str) -> &str {
    rust_path.rsplit("::").next().unwrap_or(rust_path)
}

macro_rules! root_types {
    ($($path:literal => $ty:ty),+ $(,)?) => {
        /// Every root type, sorted by `rust_path`; test `M-2` enforces the sort.
        #[must_use]
        pub fn root_types() -> Vec<RootType> {
            vec![$(RootType {
                rust_path: $path,
                schema: |generator| generator.root_schema_for::<$ty>(),
                roundtrip: |value| serde_json::from_value::<$ty>(value.clone())
                    .map_err(|error| error.to_string())
                    .and_then(|typed| serde_json::to_value(&typed).map_err(|e| e.to_string())),
            }),+]
        }
    };
}

root_types! {
    "ratatoskr_document_contracts::Document"           => ratatoskr_document_contracts::Document,
    "ratatoskr_error_contracts::ErrorEnvelope"         => ratatoskr_error_contracts::ErrorEnvelope,
    "ratatoskr_event_envelope::EventEnvelope"          => ratatoskr_event_envelope::EventEnvelope,
    "ratatoskr_identifiers::BlobRef"                   => ratatoskr_identifiers::BlobRef,
    "ratatoskr_operation_contracts::OperationProgressed"
        => ratatoskr_operation_contracts::OperationProgressed,
    "ratatoskr_operation_contracts::OperationReported"
        => ratatoskr_operation_contracts::OperationReported,
    "ratatoskr_operation_contracts::OperationSnapshot"
        => ratatoskr_operation_contracts::OperationSnapshot,
    "ratatoskr_social_contracts::SocialSourceCaptured"
        => ratatoskr_social_contracts::SocialSourceCaptured,
    "ratatoskr_social_contracts::SocialSourceSnapshot"
        => ratatoskr_social_contracts::SocialSourceSnapshot,
    "ratatoskr_social_contracts::SocialSourceUpdated"
        => ratatoskr_social_contracts::SocialSourceUpdated,
}

/// `EventPayload::EVENT_TYPE` for every registered root type that is an event payload.
///
/// Kept beside [`root_types`] rather than inside [`RootType`] because only event payloads have
/// one. Metadata rule R9 compares `[contract.event].event_type` against this map, so a
/// `contracts.toml` entry cannot claim an event name the payload type does not declare.
#[must_use]
pub fn event_payload_types() -> BTreeMap<&'static str, &'static str> {
    let mut declared = BTreeMap::new();
    declared.insert(
        "ratatoskr_operation_contracts::OperationProgressed",
        <ratatoskr_operation_contracts::OperationProgressed as EventPayload>::EVENT_TYPE,
    );
    declared.insert(
        "ratatoskr_operation_contracts::OperationReported",
        <ratatoskr_operation_contracts::OperationReported as EventPayload>::EVENT_TYPE,
    );
    declared.insert(
        "ratatoskr_social_contracts::SocialSourceCaptured",
        <ratatoskr_social_contracts::SocialSourceCaptured as EventPayload>::EVENT_TYPE,
    );
    declared.insert(
        "ratatoskr_social_contracts::SocialSourceUpdated",
        <ratatoskr_social_contracts::SocialSourceUpdated as EventPayload>::EVENT_TYPE,
    );
    declared
}
