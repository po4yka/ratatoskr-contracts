//! `EntityRef`, `EntityKind` and `TenantRef` — tests I-1, I-2, I-3, I-12 and I-14.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_identifiers::{EntityKind, EntityLocalId, EntityRef, IdentifierError, TenantRef};

/// I-1.
#[test]
fn parses_uuid_and_provider_local_ids() {
    let uuid_ref = EntityRef::parse("document:018f0000-0000-7000-8000-000000000002")
        .expect("a document reference with a UUID local part is legal");
    assert_eq!(uuid_ref.kind(), &EntityKind::Document);
    assert_eq!(
        uuid_ref.local_id().as_str(),
        "018f0000-0000-7000-8000-000000000002"
    );
    assert!(uuid_ref.as_uuid().is_some());

    // README.md's `"aggregate_id": "x-post:123"`: the local part is not a UUID at all.
    let provider_ref = EntityRef::parse("x-post:123").expect("a provider local id is legal");
    assert_eq!(provider_ref.kind().as_str(), "x-post");
    assert_eq!(provider_ref.local_id().as_str(), "123");
    assert_eq!(provider_ref.as_uuid(), None);
}

/// I-2. The split is on the FIRST colon only, so a `BlobRef`-shaped local id survives.
#[test]
fn local_id_may_contain_colon() {
    let parsed = EntityRef::parse(
        "blob:sha256:0000000000000000000000000000000000000000000000000000000000000000",
    )
    .expect("the local part may contain further colons");
    assert_eq!(parsed.kind(), &EntityKind::Blob);
    assert_eq!(
        parsed.local_id().as_str(),
        "sha256:0000000000000000000000000000000000000000000000000000000000000000"
    );
    assert_eq!(parsed.to_wire(), parsed.to_string());
}

/// I-3.
#[test]
fn rejects_missing_separator() {
    assert!(matches!(
        EntityRef::parse("018f0000-0000-7000-8000-000000000002"),
        Err(IdentifierError::MissingKindSeparator { .. })
    ));
}

/// I-3.
#[test]
fn rejects_uppercase_kind() {
    assert!(matches!(
        EntityRef::parse("Document:018f0000-0000-7000-8000-000000000002"),
        Err(IdentifierError::PatternMismatch {
            type_name: "EntityKind",
            ..
        })
    ));
}

/// I-3.
#[test]
fn rejects_over_max_len() {
    // The longest legal reference is exactly `EntityRef::MAX_LEN`: 32 kind + 1 colon + 256 local.
    let at_limit = format!("{}:{}", "k".repeat(32), "a".repeat(256));
    assert_eq!(at_limit.len(), EntityRef::MAX_LEN);
    assert!(EntityRef::parse(&at_limit).is_ok());

    // One byte more in the local half.
    assert!(matches!(
        EntityRef::parse(&format!("document:{}", "a".repeat(257))),
        Err(IdentifierError::TooLong {
            type_name: "EntityLocalId",
            max: 256,
            ..
        })
    ));

    // One byte more in the kind half.
    assert!(matches!(
        EntityRef::parse(&format!("{}:123", "k".repeat(33))),
        Err(IdentifierError::PatternMismatch {
            type_name: "EntityKind",
            ..
        })
    ));
}

/// I-3.
#[test]
fn rejects_empty_local_id() {
    assert!(matches!(
        EntityRef::parse("document:"),
        Err(IdentifierError::Empty {
            type_name: "EntityLocalId"
        })
    ));
}

/// I-3. ADR-0007: one spelling per UUID identity. Both published patterns accept
/// `event:018F0000-…`, so without this rule the same event has two unequal references and the
/// `causation_id` → `EventId` join silently misses.
#[test]
fn rejects_non_canonical_uuid_local_id() {
    let published = regex::Regex::new(EntityRef::PATTERN).expect("PATTERN compiles");
    for rejected in [
        "event:018F0000-0000-7000-8000-000000000001",
        "document:018f0000-0000-7000-8000-00000000000A",
        "user:018F0000-0000-7000-8000-000000000005",
        // An unknown provider kind is not exempt: the rule is about the identity, not the kind.
        "x-post:018F0000-0000-7000-8000-000000000002",
    ] {
        assert!(
            published.is_match(rejected),
            "{rejected} matches the published pattern, which is why the Rust layer must catch it"
        );
        let error = EntityRef::parse(rejected).expect_err("a non-canonical UUID is not a spelling");
        assert!(
            matches!(error, IdentifierError::NonCanonicalUuid { .. }),
            "{rejected}: {error}"
        );
        assert!(
            error
                .to_string()
                .contains("is a UUID in non-canonical form")
        );
    }
}

/// I-3. The rule fires on UUID shape alone, so every opaque local identity survives untouched.
/// `ARCHITECTURE.md` S5.1: "provider IDs remain opaque strings".
#[test]
fn keeps_non_uuid_local_ids_opaque_and_case_sensitive() {
    for accepted in [
        "x-post:123",
        "x-post:AbC-123",
        "repository:OWNER.Name",
        "user:123",
        "blob:sha256:0000000000000000000000000000000000000000000000000000000000000000",
        // One hex digit short of a UUID: not a UUID, so neither folded nor rejected.
        "document:018F0000-0000-7000-8000-00000000000",
    ] {
        let parsed = EntityRef::parse(accepted).expect("an opaque local identity stays legal");
        assert_eq!(parsed.to_wire(), accepted, "the local part is never folded");
    }
}

/// I-12. `DOMAIN.md` invariant 6, **preserved** branch: a kind from a bounded context this build
/// has never heard of is kept verbatim so the record stays routable, ackable and loggable.
#[test]
fn preserves_unknown_entity_kind_verbatim() {
    let wire = "repository:018f0000-0000-7000-8000-000000000009";
    let parsed = EntityRef::parse(wire).expect("an unknown kind is preserved, not rejected");
    assert_eq!(parsed.kind(), &EntityKind::Other("repository".to_owned()));
    assert!(!parsed.kind().is_known());
    assert_eq!(parsed.to_wire(), wire);

    let json = serde_json::to_string(&parsed).expect("EntityRef serializes as a string");
    assert_eq!(json, format!("\"{wire}\""));
    let reparsed: EntityRef = serde_json::from_str(&json).expect("and round-trips");
    assert_eq!(reparsed, parsed);
}

/// The five kinds this build names are exactly `contracts.toml [entity_kinds].known`.
#[test]
fn known_kinds_are_sorted_and_round_trip() {
    let mut sorted = EntityKind::KNOWN.to_vec();
    sorted.sort_unstable();
    assert_eq!(sorted, EntityKind::KNOWN);

    for token in EntityKind::KNOWN {
        let kind = EntityKind::parse(token).expect("a known token parses");
        assert!(kind.is_known());
        assert_eq!(kind.as_str(), *token);
    }
}

/// I-14. `DOMAIN.md` invariant 6, **rejected explicitly** branch. Tenancy governs authorization
/// and data separation (`SECURITY.md`: identity fields need security review), so a consumer that
/// cannot understand the owner kind must stop rather than relay the record.
#[test]
fn tenant_ref_rejects_every_kind_but_user() {
    let tenant = TenantRef::parse("user:018f0000-0000-7000-8000-000000000005")
        .expect("`user` is the one permitted kind");
    assert_eq!(
        tenant.to_string(),
        "user:018f0000-0000-7000-8000-000000000005"
    );
    assert_eq!(
        tenant.as_entity_ref(),
        EntityRef::new(
            EntityKind::User,
            EntityLocalId::parse("018f0000-0000-7000-8000-000000000005")
                .expect("a canonical UUID is a legal local id"),
        )
    );

    for rejected in [
        "organisation:018f0000-0000-7000-8000-000000000005",
        "operation:018f0000-0000-7000-8000-000000000005",
        "workspace:018f0000-0000-7000-8000-000000000005",
    ] {
        let error = TenantRef::parse(rejected).expect_err("only `user` is accepted");
        assert!(matches!(
            error,
            IdentifierError::KindMismatch {
                expected: "user",
                ..
            }
        ));
        assert!(error.to_string().contains("expected entity kind \"user\""));
    }

    assert!(matches!(
        TenantRef::parse("user:not-a-uuid"),
        Err(IdentifierError::NotAUuid { .. })
    ));
    assert!(matches!(
        TenantRef::parse("018f0000-0000-7000-8000-000000000005"),
        Err(IdentifierError::MissingKindSeparator { .. })
    ));
}
