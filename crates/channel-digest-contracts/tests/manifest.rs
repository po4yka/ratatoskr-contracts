//! The channel-digest manifest: canonical bytes, cross-field validation and the read binding
//! (XR-021 CONTRACTS.md S08).

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    reason = "assertions in a test binary; indexes run on values the test itself built"
)]

use ratatoskr_channel_digest_contracts::{
    ChannelDigestContractError, ChannelDigestManifest, ChannelDigestManifestRef,
    ChannelDigestManifestSource, HEADER_DIGEST_RUN_ID, HEADER_MANIFEST_DIGEST, HEADER_OWNER_ID,
    MANIFEST_PATH_TEMPLATE, READY_PATH, manifest_path, sha256_hex,
};
use ratatoskr_identifiers::{ContentDigest, DigestAlgorithm, DigestHex, WireTimestamp};
use sha2::{Digest as _, Sha256};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/channel_digest/manifest/valid/two-channels.json"
);

/// The canonical bytes of the two-channel fixture, written out by hand from the rule: compact,
/// every object's keys in lexicographic order, no trailing newline.
const GOLDEN: &str = r#"{"digest_run_id":"018f0000-0000-7000-8000-000000000a02","manifest_ref":"channel-digest-manifest:018f0000-0000-7000-8000-000000000a01","owner":"user:018f0000-0000-7000-8000-000000000005","schema":"channel_digest_manifest.v1","sources":[{"channel_label":"Rust Weekly","channel_ref":"telegram-public-channel:018f0000-0000-7000-8000-000000000c01","content":"Release notes for the new compiler build.","content_digest":{"algorithm":"sha256","hex":"8671ca086eaa4b1baeddc49d5371ed181cce6c3042833ee2de0afb2bf9fc3786"},"message_id":"101","published_at":"2026-08-27T06:00:00Z","revision":1,"revision_ref":"channel-post-revision:018f0000-0000-7000-8000-000000000b01"},{"channel_label":"Rust Weekly","channel_ref":"telegram-public-channel:018f0000-0000-7000-8000-000000000c01","content":"Release notes for the new compiler build (corrected).","content_digest":{"algorithm":"sha256","hex":"b4f9eea6333c604dbfb9130e98695ea76662a48422ac6df833e7691fe170cc88"},"message_id":"101","published_at":"2026-08-27T06:00:00Z","revision":2,"revision_ref":"channel-post-revision:018f0000-0000-7000-8000-000000000b03"},{"channel_label":"Новости дня","channel_ref":"telegram-public-channel:018f0000-0000-7000-8000-000000000c02","content":"Сводка за утро: всё спокойно.","content_digest":{"algorithm":"sha256","hex":"8336645c309544ee251dc7b2ef202faa3999e5c296c306841afd553773ee0e6e"},"message_id":"7","published_at":"2026-08-27T07:30:00.5Z","revision":1,"revision_ref":"channel-post-revision:018f0000-0000-7000-8000-000000000b02"},{"channel_label":"Rust Weekly","channel_ref":"telegram-public-channel:018f0000-0000-7000-8000-000000000c01","content":"Tooling update: a faster linker is now the default.","content_digest":{"algorithm":"sha256","hex":"54bfaa2256622bc0bde6ce0f496b3a09164fd64d374a65dbfcbc23dbc593fae8"},"message_id":"102","published_at":"2026-08-27T09:15:00Z","revision":1,"revision_ref":"channel-post-revision:018f0000-0000-7000-8000-000000000b04"}],"window":{"end_at":"2026-08-28T00:00:00Z","start_at":"2026-08-27T00:00:00Z"}}"#;

/// SHA-256 of [`GOLDEN`], computed outside this crate.
const GOLDEN_SHA256: &str = "c7499b5226ee969d53d066f0bfe9e59c5b6abfd9b9da2624110b28e0a65cfff0";

fn fixture_text() -> String {
    std::fs::read_to_string(FIXTURE).expect("the two-channel fixture exists")
}

fn fixture() -> ChannelDigestManifest {
    serde_json::from_str(&fixture_text()).expect("the two-channel fixture is a valid manifest")
}

fn instant(raw: &str) -> WireTimestamp {
    WireTimestamp::parse(raw).expect("a canonical instant")
}

fn digest_of(content: &str) -> ContentDigest {
    ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        hex: DigestHex::parse(&hex::encode(Sha256::digest(content.as_bytes())))
            .expect("a SHA-256 digest"),
    }
}

/// One source in channel `channel`, message `message`, published `minute` minutes after the
/// window start. Its `revision_ref` is derived from `index`.
fn source(
    index: u32,
    channel: u32,
    message: &str,
    revision: u32,
    minute: u32,
) -> ChannelDigestManifestSource {
    let content = format!("post {index} of channel {channel}");
    ChannelDigestManifestSource {
        revision_ref: format!("channel-post-revision:018f0000-0000-7000-8000-{index:012x}"),
        channel_ref: format!("telegram-public-channel:018f0000-0000-7000-8000-{channel:012x}"),
        channel_label: format!("Channel {channel}"),
        message_id: message.to_owned(),
        published_at: instant(&format!(
            "2026-08-27T{:02}:{:02}:00Z",
            minute / 60,
            minute % 60
        )),
        content_digest: digest_of(&content),
        content,
        public_link: None,
        revision,
    }
}

fn with_sources(sources: Vec<ChannelDigestManifestSource>) -> ChannelDigestManifest {
    ChannelDigestManifest {
        sources,
        ..fixture()
    }
}

#[test]
fn canonical_bytes_are_key_sorted_compact_and_stable() {
    let manifest = fixture();

    let bytes = manifest
        .to_canonical_bytes()
        .expect("a valid manifest encodes");
    assert_eq!(
        String::from_utf8(bytes.clone()).expect("canonical bytes are UTF-8"),
        GOLDEN
    );
    assert!(!bytes.ends_with(b"\n"), "no trailing newline");
    assert_eq!(sha256_hex(&bytes), GOLDEN_SHA256);
    assert_eq!(
        manifest.to_canonical_bytes().expect("encodes again"),
        bytes,
        "the encoding is stable"
    );

    // Declaration order is not key order, so the sorting is doing work.
    let declaration_order = serde_json::to_vec(&manifest).expect("serializes");
    assert_ne!(declaration_order, bytes);

    let decoded =
        ChannelDigestManifest::from_canonical_bytes(&bytes).expect("canonical bytes decode");
    assert_eq!(decoded, manifest);
}

#[test]
fn from_canonical_bytes_rejects_reordered_or_pretty_bytes_as_integrity() {
    let manifest = fixture();
    let canonical = manifest.to_canonical_bytes().expect("encodes");

    let mut newline = canonical.clone();
    newline.push(b'\n');
    for (label, bytes) in [
        ("pretty declaration order", fixture_text().into_bytes()),
        (
            "compact declaration order",
            serde_json::to_vec(&manifest).expect("serializes"),
        ),
        ("trailing newline", newline),
    ] {
        assert_eq!(
            ChannelDigestManifest::from_canonical_bytes(&bytes),
            Err(ChannelDigestContractError::ManifestIntegrity),
            "{label} must be an integrity failure"
        );
    }

    let mut with_null_link = serde_json::from_slice::<serde_json::Value>(&canonical).unwrap();
    with_null_link["sources"][2]["public_link"] = serde_json::Value::Null;
    let null_bytes = serde_json::to_vec(&with_null_link).unwrap();
    assert_eq!(
        ChannelDigestManifest::from_canonical_bytes(&null_bytes),
        Err(ChannelDigestContractError::ManifestIntegrity),
        "an explicit null is not the canonical spelling of an absent link"
    );

    for (label, bytes) in [
        ("not JSON", b"not json".to_vec()),
        ("an empty body", Vec::new()),
        (
            "an unknown member",
            GOLDEN
                .replacen("{\"digest_run_id\"", "{\"aaa\":1,\"digest_run_id\"", 1)
                .into_bytes(),
        ),
        (
            "another schema marker",
            GOLDEN
                .replace("channel_digest_manifest.v1", "channel_digest_manifest.v2")
                .into_bytes(),
        ),
    ] {
        assert_eq!(
            ChannelDigestManifest::from_canonical_bytes(&bytes),
            Err(ChannelDigestContractError::ManifestEncoding),
            "{label} must be an encoding failure"
        );
    }
}

#[test]
fn validate_rejects_unsorted_sources() {
    fixture().validate().expect("the fixture is valid");

    let invalid = ChannelDigestContractError::ManifestInvalid;

    let mut unsorted = fixture();
    unsorted.sources.swap(0, 2);
    assert_eq!(unsorted.validate(), Err(invalid), "sources out of order");

    let mut outside_window = fixture();
    outside_window.sources[3].published_at = outside_window.window.end_at;
    assert_eq!(
        outside_window.validate(),
        Err(invalid),
        "the window end is exclusive"
    );
    let mut before_window = fixture();
    before_window.sources[0].published_at = instant("2026-08-26T23:59:59Z");
    assert_eq!(before_window.validate(), Err(invalid), "before the window");
    let mut at_start = fixture();
    at_start.sources[0].published_at = at_start.window.start_at;
    at_start.validate().expect("the window start is inclusive");

    let mut duplicate_ref = fixture();
    duplicate_ref.sources[3].revision_ref = duplicate_ref.sources[0].revision_ref.clone();
    assert_eq!(
        duplicate_ref.validate(),
        Err(invalid),
        "duplicate revision_ref"
    );

    let mut tampered_digest = fixture();
    tampered_digest.sources[3].content_digest = digest_of("something else entirely");
    assert_eq!(
        tampered_digest.validate(),
        Err(invalid),
        "bad content_digest"
    );

    let mut query_link = fixture();
    query_link.sources[3].public_link = Some("https://t.me/rust_weekly/102?single".to_owned());
    assert_eq!(
        query_link.validate(),
        Err(invalid),
        "link with a query string"
    );
    let mut fragment_link = fixture();
    fragment_link.sources[3].public_link = Some("https://t.me/rust_weekly/102#top".to_owned());
    assert_eq!(
        fragment_link.validate(),
        Err(invalid),
        "link with a fragment"
    );
    let mut foreign_link = fixture();
    foreign_link.sources[3].public_link = Some("https://example.org/rust_weekly/102".to_owned());
    assert_eq!(
        foreign_link.validate(),
        Err(invalid),
        "link to another host"
    );
    let mut long_link = fixture();
    long_link.sources[3].public_link = Some(format!("https://t.me/{}/102", "a".repeat(600)));
    assert_eq!(long_link.validate(), Err(invalid), "link longer than 512");

    assert_eq!(
        with_sources(Vec::new()).validate(),
        Err(invalid),
        "no sources"
    );
    let mut zero_revision = fixture();
    zero_revision.sources[0].revision = 0;
    assert_eq!(
        zero_revision.validate(),
        Err(invalid),
        "revisions start at 1"
    );
    let mut long_label = fixture();
    long_label.sources[0].channel_label = "x".repeat(81);
    assert_eq!(long_label.validate(), Err(invalid), "label longer than 80");
    let mut empty_label = fixture();
    empty_label.sources[0].channel_label = String::new();
    assert_eq!(empty_label.validate(), Err(invalid), "empty label");
    let mut word_id = fixture();
    word_id.sources[0].message_id = "one".to_owned();
    assert_eq!(word_id.validate(), Err(invalid), "message_id is decimal");
    let mut wrong_ref = fixture();
    wrong_ref.sources[0].channel_ref = "telegram-channel:018f0000".to_owned();
    assert_eq!(wrong_ref.validate(), Err(invalid), "channel_ref grammar");
}

#[test]
fn a_canonical_public_link_is_accepted_and_sorts_between_message_id_and_published_at() {
    let mut linked = fixture();
    linked.sources[0].public_link = Some("https://t.me/rust_weekly/101".to_owned());
    linked.sources[1].public_link = Some("https://t.me/rust_weekly/101".to_owned());
    linked.sources[3].public_link = Some("https://t.me/rust_weekly/102".to_owned());
    linked
        .validate()
        .expect("a canonical t.me link is acceptable");

    let bytes = linked.to_canonical_bytes().expect("encodes");
    let text = String::from_utf8(bytes.clone()).expect("UTF-8");
    assert!(
        text.contains(
            r#""message_id":"102","public_link":"https://t.me/rust_weekly/102","published_at":"#
        ),
        "keys are sorted: {text}"
    );
    assert_eq!(
        ChannelDigestManifest::from_canonical_bytes(&bytes).expect("decodes"),
        linked
    );
}

#[test]
fn validate_enforces_the_size_bounds() {
    let invalid = ChannelDigestContractError::ManifestInvalid;

    let channels = |count: u32| {
        (0..count)
            .map(|index| source(index + 1, index + 1, "1", 1, index))
            .collect::<Vec<_>>()
    };
    with_sources(channels(20))
        .validate()
        .expect("twenty channels are the limit");
    assert_eq!(
        with_sources(channels(21)).validate(),
        Err(invalid),
        "twenty-one channels"
    );

    let messages = |count: u32| {
        (0..count)
            .map(|index| source(index + 1, 1, &(index + 1).to_string(), 1, index % 1_000))
            .collect::<Vec<_>>()
    };
    with_sources(messages(100))
        .validate()
        .expect("one hundred sources are the limit");
    assert_eq!(
        with_sources(messages(101)).validate(),
        Err(invalid),
        "one hundred and one sources"
    );

    let mut at_limit = source(1, 1, "1", 1, 0);
    at_limit.content = "a".repeat(16_384);
    at_limit.content_digest = digest_of(&at_limit.content);
    with_sources(vec![at_limit.clone()])
        .validate()
        .expect("16384 bytes are the limit");
    at_limit.content.push('a');
    at_limit.content_digest = digest_of(&at_limit.content);
    assert_eq!(
        with_sources(vec![at_limit]).validate(),
        Err(invalid),
        "16385 bytes"
    );

    // The limit counts UTF-8 bytes, not characters: 8193 two-byte characters are 16386 bytes.
    let mut wide = source(1, 1, "1", 1, 0);
    wide.content = "я".repeat(8_193);
    wide.content_digest = digest_of(&wide.content);
    assert_eq!(
        with_sources(vec![wide]).validate(),
        Err(invalid),
        "bytes, not characters"
    );

    let mut empty = source(1, 1, "1", 1, 0);
    empty.content = String::new();
    empty.content_digest = digest_of("");
    assert_eq!(
        with_sources(vec![empty]).validate(),
        Err(invalid),
        "empty content"
    );
}

#[test]
fn message_identity_is_compared_as_a_decimal_integer() {
    let invalid = ChannelDigestContractError::ManifestInvalid;
    let ascending = with_sources(vec![source(1, 1, "9", 1, 1), source(2, 1, "10", 1, 1)]);
    ascending
        .validate()
        .expect("9 sorts before 10 when compared as integers");
    let descending = with_sources(vec![source(1, 1, "10", 1, 1), source(2, 1, "9", 1, 1)]);
    assert_eq!(
        descending.validate(),
        Err(invalid),
        "10 before 9 is out of order although it is in string order"
    );

    let same_identity = with_sources(vec![source(1, 1, "7", 1, 1), source(2, 1, "007", 1, 1)]);
    assert_eq!(
        same_identity.validate(),
        Err(invalid),
        "7 and 007 are one message identity"
    );
}

#[test]
fn deserialize_runs_the_cross_field_rules() {
    let mut unsorted = fixture();
    unsorted.sources.swap(0, 2);
    let value = serde_json::to_value(&unsorted).expect("serializes");
    let error = serde_json::from_value::<ChannelDigestManifest>(value)
        .expect_err("an unsorted manifest does not decode");
    assert!(error.to_string().contains("cross-field rule"), "{error}");
}

#[test]
fn manifest_path_and_header_constants_match_the_documented_values() {
    assert_eq!(MANIFEST_PATH_TEMPLATE, "/v1/manifests/{manifest_id}");
    assert_eq!(READY_PATH, "/ready");
    assert_eq!(HEADER_OWNER_ID, "x-ratatoskr-owner-id");
    assert_eq!(HEADER_DIGEST_RUN_ID, "x-ratatoskr-digest-run-id");
    assert_eq!(HEADER_MANIFEST_DIGEST, "x-ratatoskr-manifest-digest");

    let reference = ChannelDigestManifestRef::parse(
        "channel-digest-manifest:018f0000-0000-7000-8000-000000000a01",
    )
    .expect("a legal manifest reference");
    assert_eq!(
        manifest_path(&reference),
        "/v1/manifests/018f0000-0000-7000-8000-000000000a01"
    );

    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
