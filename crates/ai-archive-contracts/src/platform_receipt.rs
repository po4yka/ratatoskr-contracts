//! The loopback archive receipt binding between Platform's Edge and the `ratatoskr-chatgpt` and
//! `ratatoskr-claude` receivers (XR-021 CONTRACTS.md S06 D1 to D3).
//!
//! Not a registered wire message: these are the literals both sides of a loopback HTTP hop must
//! spell identically, so each side imports them and asserts against them instead of copying
//! strings. `contracts.toml` and `schemas/` do not describe this module.
//!
//! # Receipt request
//!
//! Edge sends `POST` [`RECEIPT_PATH`] to the receiver's loopback listener. `Content-Type` is
//! exactly [`ARCHIVE_MEDIA_TYPE`] with no parameters and `Content-Length` equals the declared byte
//! size. Edge mints six claims: [`HEADER_USER_ID`] and [`HEADER_DEVICE_ID`] (UUIDs),
//! [`HEADER_CORRELATION_ID`] (1 to 200 characters), [`HEADER_OPERATION_ID`] (UUID),
//! [`HEADER_ARCHIVE_SHA256`] (64 lowercase hexadecimal characters) and
//! [`HEADER_ARCHIVE_BYTE_SIZE`] (decimal `u64`, greater than zero). There is no `Authorization`
//! header and no `Cookie`. The body is the raw archive, streamed in frames and never buffered
//! whole. Any 2xx answer is success; both receivers answer `202 Accepted` with an empty body.
//! Every non-2xx answer is `application/json` carrying an `ErrorEnvelope` with a non-empty code,
//! because Edge replaces anything else with `edge.upstream_invalid_response`.
//!
//! # Capability probe
//!
//! `GET` [`CAPABILITIES_PATH`] needs no claims and exists only while the receipt route is
//! mounted. It answers `200 application/json`, at most 65536 bytes, with the document built by
//! [`receipt_capability_document`]. Edge accepts a receiver as ready only when
//! [`is_receipt_capability_document`] passes for the route name, so an unrelated service on the
//! same port is never mistaken for a receiver.
//!
//! # Incomplete imports
//!
//! A producer that imported an archive whose completeness is not `complete` reports
//! `partially_succeeded` with exactly one warning, [`incomplete_import_warning`], and no error.

use ratatoskr_error_contracts::{ErrorCode, WarningEnvelope};
use ratatoskr_identifiers::{Extensions, SafeMessage};

/// Path of the loopback receipt route on a receiver's listener.
pub const RECEIPT_PATH: &str = "/v1/ai-archives/receipt";

/// The only accepted `Content-Type` of a receipt request, without parameters.
pub const ARCHIVE_MEDIA_TYPE: &str = "application/zip";

/// Edge-minted claim: the Platform user (UUID).
pub const HEADER_USER_ID: &str = "x-ratatoskr-user-id";

/// Edge-minted claim: the device that uploaded the archive (UUID).
pub const HEADER_DEVICE_ID: &str = "x-ratatoskr-device-id";

/// Edge-minted claim: the request correlation id (1 to 200 characters).
pub const HEADER_CORRELATION_ID: &str = "x-correlation-id";

/// Edge-minted claim: the Platform operation the upload belongs to (UUID).
pub const HEADER_OPERATION_ID: &str = "x-ratatoskr-operation-id";

/// Edge-minted claim: SHA-256 of the archive bytes (64 lowercase hexadecimal characters).
pub const HEADER_ARCHIVE_SHA256: &str = "x-ratatoskr-archive-sha256";

/// Edge-minted claim: exact archive size in bytes (decimal `u64`, greater than zero).
pub const HEADER_ARCHIVE_BYTE_SIZE: &str = "x-ratatoskr-archive-byte-size";

/// Path of the unauthenticated capability probe on a receiver's listener.
pub const CAPABILITIES_PATH: &str = "/v1/capabilities";

/// The capability a receiver advertises while the receipt route is mounted.
pub const RECEIPT_CAPABILITY: &str = "ai_archive.receipt";

/// Code of the warning that marks an imported but incomplete archive.
pub const INCOMPLETE_IMPORT_WARNING_CODE: &str = "ai_archive.import.incomplete";

/// Message of the warning that marks an imported but incomplete archive.
pub const INCOMPLETE_IMPORT_WARNING_MESSAGE: &str =
    "The archive was imported, but it is not complete.";

/// The capability document a receiver serves at [`CAPABILITIES_PATH`]:
/// `{"service": <service>, "capabilities": ["ai_archive.receipt"]}`.
///
/// `service` is the Edge route name, `chatgpt` or `claude`.
#[must_use]
pub fn receipt_capability_document(service: &str) -> serde_json::Value {
    serde_json::json!({
        "service": service,
        "capabilities": [RECEIPT_CAPABILITY],
    })
}

/// Whether `document` is a receiver's capability document for `service`.
///
/// True only when `service` is a string equal to the expected route name and `capabilities` is an
/// array that contains [`RECEIPT_CAPABILITY`]. Further capabilities and unknown members are
/// tolerated.
#[must_use]
pub fn is_receipt_capability_document(document: &serde_json::Value, service: &str) -> bool {
    document
        .get("service")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|named| named == service)
        && document
            .get("capabilities")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|capabilities| {
                capabilities
                    .iter()
                    .any(|capability| capability.as_str() == Some(RECEIPT_CAPABILITY))
            })
}

/// The single warning that accompanies `partially_succeeded` for an incomplete import.
///
/// Carries [`INCOMPLETE_IMPORT_WARNING_CODE`] and [`INCOMPLETE_IMPORT_WARNING_MESSAGE`], no
/// `field_path` and no extensions.
///
/// # Panics
///
/// Only if one of the two static spellings above stops satisfying its grammar, which the
/// `incomplete_import_warning_is_a_valid_warning_envelope` test guards as a contract regression.
#[must_use]
#[allow(
    clippy::expect_used,
    reason = "the static spellings are covered by the contract test"
)]
pub fn incomplete_import_warning() -> WarningEnvelope {
    WarningEnvelope {
        code: ErrorCode::parse(INCOMPLETE_IMPORT_WARNING_CODE)
            .expect("the static warning code satisfies the ErrorCode grammar"),
        message: SafeMessage::parse(INCOMPLETE_IMPORT_WARNING_MESSAGE)
            .expect("the static warning message satisfies the SafeMessage grammar"),
        field_path: None,
        extensions: Extensions::new(),
    }
}
