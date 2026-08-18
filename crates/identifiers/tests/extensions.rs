//! `Extensions` — tests I-11 and I-13. `DOMAIN.md` invariant 6, **preserved** branch.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in a test binary"
)]

use ratatoskr_identifiers::{EventId, Extensions, canonical_json, dropped_field_pointers};

/// A stand-in for any tolerant wire struct: a known member plus the flattened preserve map.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct TolerantProbe {
    event_id: EventId,
    #[serde(flatten)]
    extensions: Extensions,
}

/// I-11. An empty preserve map leaves no `{}` artifact on the wire.
#[test]
fn empty_extensions_serialize_to_nothing() {
    let probe = TolerantProbe {
        event_id: EventId::parse("018f0000-0000-7000-8000-000000000001").unwrap(),
        extensions: Extensions::new(),
    };
    assert!(probe.extensions.is_empty());
    assert_eq!(probe.extensions.len(), 0);
    assert_eq!(
        serde_json::to_string(&probe).unwrap(),
        r#"{"event_id":"018f0000-0000-7000-8000-000000000001"}"#
    );
    assert_eq!(Extensions::new(), Extensions::default());
}

/// I-13. Preserved keys re-emit in sorted order, so a tolerant parse is still byte-deterministic.
#[test]
fn preserved_keys_reemit_in_sorted_order() {
    let mut extensions = Extensions::new();
    assert_eq!(extensions.insert("zeta_count", serde_json::json!(1)), None);
    extensions.insert("alpha_label", serde_json::json!("a"));
    extensions.insert("mu_flag", serde_json::json!(true));
    assert_eq!(
        extensions.keys().collect::<Vec<_>>(),
        vec!["alpha_label", "mu_flag", "zeta_count"]
    );
    assert_eq!(extensions.get("mu_flag"), Some(&serde_json::json!(true)));
    assert_eq!(extensions.as_map().len(), 3);

    let probe = TolerantProbe {
        event_id: EventId::parse("018f0000-0000-7000-8000-000000000001").unwrap(),
        extensions,
    };
    assert_eq!(
        serde_json::to_string(&probe).unwrap(),
        concat!(
            r#"{"event_id":"018f0000-0000-7000-8000-000000000001","#,
            r#""alpha_label":"a","mu_flag":true,"zeta_count":1}"#
        )
    );
}

/// A member this build has never heard of survives parse and re-emit with nothing dropped.
#[test]
fn unknown_members_round_trip_losslessly() {
    let wire = concat!(
        r#"{"event_id":"018f0000-0000-7000-8000-000000000001","#,
        r#""retention_class":{"nested_count":2,"labels":["a","b"]}}"#
    );
    let probe: TolerantProbe = serde_json::from_str(wire).expect("unknown members are preserved");
    assert_eq!(probe.extensions.len(), 1);

    let input: serde_json::Value = serde_json::from_str(wire).unwrap();
    let roundtripped: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&probe).unwrap()).unwrap();
    assert_eq!(
        dropped_field_pointers(&input, &roundtripped),
        Vec::<String>::new()
    );
    assert!(canonical_json(&probe).unwrap().ends_with("}\n"));
}

/// `dropped_field_pointers` names what was lost, with RFC 6901 escaping.
#[test]
fn dropped_field_pointers_report_every_lost_member() {
    let input = serde_json::json!({
        "kept": 1,
        "lost": 2,
        "nested": {"kept": 1, "lost/slash": 2, "lost~tilde": 3},
        "items": [{"kept": 1}, {"lost": 2}]
    });
    let roundtripped = serde_json::json!({
        "kept": 1,
        "nested": {"kept": 1},
        "items": [{"kept": 1}]
    });
    assert_eq!(
        dropped_field_pointers(&input, &roundtripped),
        vec![
            "/items/1".to_owned(),
            "/lost".to_owned(),
            "/nested/lost~1slash".to_owned(),
            "/nested/lost~0tilde".to_owned(),
        ]
    );
    assert!(dropped_field_pointers(&input, &input).is_empty());
}
