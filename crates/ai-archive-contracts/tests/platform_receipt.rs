//! The loopback archive receipt binding shared by Platform and the two archive receivers
//! (XR-021 CONTRACTS.md S06 D1 to D3).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_ai_archive_contracts::platform_receipt::{
    ARCHIVE_MEDIA_TYPE, CAPABILITIES_PATH, HEADER_ARCHIVE_BYTE_SIZE, HEADER_ARCHIVE_SHA256,
    HEADER_CORRELATION_ID, HEADER_DEVICE_ID, HEADER_OPERATION_ID, HEADER_USER_ID,
    INCOMPLETE_IMPORT_WARNING_CODE, INCOMPLETE_IMPORT_WARNING_MESSAGE, RECEIPT_CAPABILITY,
    RECEIPT_PATH, incomplete_import_warning, is_receipt_capability_document,
    receipt_capability_document,
};
use ratatoskr_error_contracts::WarningEnvelope;
use serde_json::json;

#[test]
fn binding_constants_are_the_documented_literals() {
    assert_eq!(RECEIPT_PATH, "/v1/ai-archives/receipt");
    assert_eq!(ARCHIVE_MEDIA_TYPE, "application/zip");
    assert_eq!(HEADER_USER_ID, "x-ratatoskr-user-id");
    assert_eq!(HEADER_DEVICE_ID, "x-ratatoskr-device-id");
    assert_eq!(HEADER_CORRELATION_ID, "x-correlation-id");
    assert_eq!(HEADER_OPERATION_ID, "x-ratatoskr-operation-id");
    assert_eq!(HEADER_ARCHIVE_SHA256, "x-ratatoskr-archive-sha256");
    assert_eq!(HEADER_ARCHIVE_BYTE_SIZE, "x-ratatoskr-archive-byte-size");
    assert_eq!(CAPABILITIES_PATH, "/v1/capabilities");
    assert_eq!(RECEIPT_CAPABILITY, "ai_archive.receipt");
    assert_eq!(
        INCOMPLETE_IMPORT_WARNING_CODE,
        "ai_archive.import.incomplete"
    );
    assert_eq!(
        INCOMPLETE_IMPORT_WARNING_MESSAGE,
        "The archive was imported, but it is not complete."
    );
}

#[test]
fn capability_document_round_trips_and_names_the_service() {
    for service in ["chatgpt", "claude"] {
        let document = receipt_capability_document(service);
        assert_eq!(
            document,
            json!({"service": service, "capabilities": ["ai_archive.receipt"]})
        );

        let bytes = serde_json::to_vec(&document).expect("the document serializes");
        assert!(bytes.len() <= 65_536, "the probe answer stays small");
        let parsed: serde_json::Value =
            serde_json::from_slice(&bytes).expect("the document parses");
        assert!(is_receipt_capability_document(&parsed, service));
    }
}

#[test]
fn a_document_for_another_service_or_without_the_capability_is_rejected() {
    let chatgpt = receipt_capability_document("chatgpt");
    assert!(!is_receipt_capability_document(&chatgpt, "claude"));
    assert!(!is_receipt_capability_document(&chatgpt, "github"));

    for rejected in [
        json!({"service": "chatgpt", "capabilities": []}),
        json!({"service": "chatgpt", "capabilities": ["repository_preview"]}),
        json!({"service": "chatgpt"}),
        json!({"capabilities": ["ai_archive.receipt"]}),
        json!({"service": 7, "capabilities": ["ai_archive.receipt"]}),
        json!({"service": "chatgpt", "capabilities": "ai_archive.receipt"}),
        json!(["chatgpt", "ai_archive.receipt"]),
        json!(null),
    ] {
        assert!(
            !is_receipt_capability_document(&rejected, "chatgpt"),
            "{rejected} must not pass"
        );
    }

    let wider = json!({"service": "chatgpt", "capabilities": ["other", "ai_archive.receipt"]});
    assert!(
        is_receipt_capability_document(&wider, "chatgpt"),
        "a service may advertise more capabilities beside the receipt"
    );
}

#[test]
fn incomplete_import_warning_is_a_valid_warning_envelope() {
    let warning = incomplete_import_warning();
    let encoded = serde_json::to_value(&warning).expect("the warning serializes");

    assert_eq!(
        encoded,
        json!({
            "code": "ai_archive.import.incomplete",
            "message": "The archive was imported, but it is not complete."
        }),
        "no field_path and no extensions"
    );
    let decoded: WarningEnvelope = serde_json::from_value(encoded).expect("the warning parses");
    assert_eq!(decoded, warning);
}
