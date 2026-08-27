//! Typed UUID identities for the fields that carry a record's own identity (ADR-0007).

use core::convert::TryFrom;
use core::fmt;
use core::str::FromStr;

use crate::entity::{EntityKind, EntityLocalId, EntityRef};
use crate::error::IdentifierError;

/// The one canonical textual UUID form this repository accepts on the wire.
pub(crate) const UUID_PATTERN: &str =
    r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

/// `UUID_PATTERN`, compiled once.
#[allow(
    clippy::expect_used,
    reason = "UUID_PATTERN is a compile-time contract constant; a build whose pattern does not \
              compile is broken before it reaches a wire"
)]
fn compiled_uuid_pattern() -> &'static regex::Regex {
    static COMPILED: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(UUID_PATTERN).expect("UUID_PATTERN must be a valid regular expression")
    });
    &COMPILED
}

/// `Some` only for a canonical lowercase hyphenated UUID.
///
/// `uuid::Uuid::parse_str` also accepts uppercase, braced, `urn:`-prefixed and unhyphenated text;
/// admitting those would give one identity several spellings, so the pattern is checked first.
pub(crate) fn canonical_uuid(raw: &str) -> Option<uuid::Uuid> {
    if !compiled_uuid_pattern().is_match(raw) {
        return None;
    }
    uuid::Uuid::parse_str(raw).ok()
}

macro_rules! uuid_newtype {
    (
        $(#[doc = $doc:literal])*
        pub struct $name:ident, kind = $kind:literal, description = $description:literal
    ) => {
        $(#[doc = $doc])*
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            serde::Serialize,
            serde::Deserialize,
        )]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(pub uuid::Uuid);

        impl $name {
            /// The entity kind this identity uses inside an [`EntityRef`].
            pub const KIND: &'static str = $kind;

            /// The published JSON Schema `pattern`: the canonical lowercase hyphenated form only.
            pub const PATTERN: &'static str = crate::uuid_ids::UUID_PATTERN;

            /// Mints a new time-ordered `UUIDv7` (`ARCHITECTURE.md` S5.1).
            /// Never called by the generator or by fixtures.
            #[must_use]
            pub fn new_v7() -> Self {
                Self(uuid::Uuid::now_v7())
            }

            /// Parses the bare canonical UUID text.
            ///
            /// # Errors
            ///
            /// [`IdentifierError::PatternMismatch`] for uppercase, braced, `urn:`-prefixed or
            /// unhyphenated input. `uuid::Uuid::try_parse` accepts all of those; rejecting them
            /// is what makes the canonical text unique, so `PATTERN` is checked **before**
            /// `try_parse`.
            pub fn parse(raw: &str) -> Result<Self, IdentifierError> {
                match crate::uuid_ids::canonical_uuid(raw) {
                    Some(value) => Ok(Self(value)),
                    None => Err(IdentifierError::PatternMismatch {
                        type_name: stringify!($name),
                        pattern: Self::PATTERN,
                        input: raw.to_owned(),
                    }),
                }
            }

            /// Whether this identity was minted as a `UUIDv7`. Deserialization accepts any RFC 9562
            /// UUID so identities minted before the v7 rule stay replayable; only *new*
            /// identities must be v7.
            #[must_use]
            pub fn is_uuid_v7(&self) -> bool {
                self.0.get_version_num() == 7
            }

            /// The same identity widened to the open reference vocabulary.
            #[must_use]
            pub fn as_entity_ref(&self) -> EntityRef {
                EntityRef::new(EntityKind::from_token(Self::KIND), EntityLocalId::from_uuid(self.0))
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }

        impl FromStr for $name {
            type Err = IdentifierError;

            fn from_str(raw: &str) -> Result<Self, Self::Err> {
                Self::parse(raw)
            }
        }

        impl TryFrom<String> for $name {
            type Error = IdentifierError;

            fn try_from(raw: String) -> Result<Self, Self::Error> {
                Self::parse(&raw)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0.to_string()
            }
        }

        impl From<$name> for EntityRef {
            fn from(value: $name) -> Self {
                value.as_entity_ref()
            }
        }

        impl TryFrom<&EntityRef> for $name {
            type Error = IdentifierError;

            fn try_from(value: &EntityRef) -> Result<Self, Self::Error> {
                if value.kind().as_str() != Self::KIND {
                    return Err(IdentifierError::KindMismatch {
                        expected: Self::KIND,
                        actual: value.kind().as_str().to_owned(),
                    });
                }
                match value.as_uuid() {
                    Some(inner) => Ok(Self(inner)),
                    None => Err(IdentifierError::NotAUuid {
                        local_id: value.local_id().as_str().to_owned(),
                    }),
                }
            }
        }

        impl schemars::JsonSchema for $name {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(stringify!($name))
            }

            fn schema_id() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(concat!(module_path!(), "::", stringify!($name)))
            }

            fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
                schemars::json_schema!({
                    "type": "string",
                    "format": "uuid",
                    "title": stringify!($name),
                    "description": $description,
                    "pattern": Self::PATTERN,
                    "examples": ["018f0000-0000-7000-8000-000000000001"],
                })
            }
        }
    };
}

uuid_newtype! {
    /// Identity of one typed block within one immutable Document IR revision.
    ///
    /// Wire form: bare canonical lowercase hyphenated UUID. The producer derives the value
    /// deterministically from the immutable revision and block position; it is not a standalone
    /// domain record identity.
    pub struct BlockId,
    kind = "block",
    description = "Identity of one typed block within one immutable Document IR revision. Bare \
                   canonical lowercase hyphenated UUID; stable only within that revision."
}

uuid_newtype! {
    /// Identity of one normalized document.
    ///
    /// Wire form: bare canonical lowercase hyphenated UUID.
    pub struct DocumentId,
    kind = "document",
    description = "Identity of one normalized document. Bare canonical lowercase hyphenated UUID; \
                   not namespaced."
}

uuid_newtype! {
    /// Identity of one event occurrence. Also the at-least-once **deduplication key**: a consumer
    /// that has already processed this `event_id` must treat a redelivery as a no-op
    /// (`ARCHITECTURE.md` S15.6).
    ///
    /// Wire form: bare canonical lowercase hyphenated UUID, e.g.
    /// `018f0000-0000-7000-8000-000000000001`. Not namespaced — see the crate-level wire-form
    /// rule.
    pub struct EventId,
    kind = "event",
    description = "Identity of one event occurrence, and the at-least-once deduplication key. \
                   Bare canonical lowercase hyphenated UUID; not namespaced."
}

uuid_newtype! {
    /// Identity of one asynchronous command delivery. A consumer uses it to deduplicate the same
    /// delivery; a command payload carries any separate domain idempotency key.
    ///
    /// Wire form: bare canonical lowercase hyphenated UUID.
    pub struct CommandId,
    kind = "command",
    description = "Identity of one asynchronous command delivery. Bare canonical lowercase \
                   hyphenated UUID; not namespaced."
}

uuid_newtype! {
    /// Identity of a Ratatoskr end user. Producer-asserted.
    ///
    /// Wire form: bare canonical lowercase hyphenated UUID. Inside an owner field it appears
    /// namespaced as [`TenantRef`]; on its own it is bare — see the crate-level wire-form rule.
    ///
    /// [`TenantRef`]: crate::TenantRef
    pub struct UserId,
    kind = "user",
    description = "Identity of a Ratatoskr end user. Bare canonical lowercase hyphenated UUID; \
                   not namespaced."
}

uuid_newtype! {
    /// Identity of one long-running operation (`ARCHITECTURE.md` S5.4). Minted by the service
    /// that accepts the request, stable for the whole lifecycle.
    ///
    /// Wire form: bare canonical lowercase hyphenated UUID.
    pub struct OperationId,
    kind = "operation",
    description = "Identity of one long-running operation, stable for its whole lifecycle. Bare \
                   canonical lowercase hyphenated UUID; not namespaced."
}

uuid_newtype! {
    /// Identity of one normalized social source — a post, permalink or capture preserved from
    /// X, Instagram or Threads into a user's library.
    ///
    /// Wire form: bare canonical lowercase hyphenated UUID. The provider's own id for the same
    /// post is a different field (`external_post_id`) and stays opaque; this identity is
    /// Ratatoskr's own and is never derived from the provider's.
    pub struct SocialSourceId,
    kind = "social_source",
    description = "Identity of one normalized social source in a user's library. Bare canonical \
                   lowercase hyphenated UUID; not namespaced."
}

uuid_newtype! {
    /// Identity of one repository record in the GitHub Catalog.
    ///
    /// Wire form: bare canonical lowercase hyphenated UUID. This is Ratatoskr's catalog
    /// identity, distinct from GitHub's stable numeric repository identifier.
    pub struct RepositoryId,
    kind = "repository",
    description = "Identity of one repository record in the GitHub Catalog. Bare canonical \
                   lowercase hyphenated UUID; not GitHub's numeric repository identifier."
}

uuid_newtype! {
    /// Identity of one repository-analysis request accepted by Knowledge.
    ///
    /// Wire form: bare canonical lowercase hyphenated UUID. It is the terminal-fact join key,
    /// not the transport envelope's at-least-once event identity.
    pub struct RepositoryAnalysisRequestId,
    kind = "repository_analysis_request",
    description = "Identity of one repository-analysis request. Bare canonical lowercase \
                   hyphenated UUID; used to join terminal facts to the accepted request."
}

uuid_newtype! {
    /// Identity of one import of a provider AI account export — one parse of the immutable
    /// evidence into normalized graph nodes.
    ///
    /// Wire form: bare canonical lowercase hyphenated UUID. The provider's own export/session
    /// identifiers stay opaque in their own fields; this identity is Ratatoskr's own and is
    /// never derived from the provider's.
    pub struct AiArchiveId,
    kind = "ai_archive",
    description = "Identity of one import of a provider AI account export. Bare canonical \
                   lowercase hyphenated UUID; not namespaced."
}

uuid_newtype! {
    /// Identity of one normalized AI project.
    ///
    /// Wire form: bare canonical lowercase hyphenated UUID. The provider's own project id is a
    /// different field (`external_project_id`) and stays opaque; this identity is Ratatoskr's
    /// own and is never derived from the provider's.
    pub struct AiProjectId,
    kind = "ai_project",
    description = "Identity of one normalized AI project. Bare canonical lowercase hyphenated \
                   UUID; not namespaced."
}

uuid_newtype! {
    /// Identity of one normalized AI conversation.
    ///
    /// Wire form: bare canonical lowercase hyphenated UUID. The provider's own conversation id
    /// is a different field (`external_conversation_id`) and stays opaque; this identity is
    /// Ratatoskr's own and is never derived from the provider's.
    pub struct AiConversationId,
    kind = "ai_conversation",
    description = "Identity of one normalized AI conversation. Bare canonical lowercase \
                   hyphenated UUID; not namespaced."
}

uuid_newtype! {
    /// Identity of one raised user-facing notification — one producer judgment that a user
    /// should be told something, carried as `platform.notification.raised.v1`.
    ///
    /// Wire form: bare canonical lowercase hyphenated UUID. Also the logical key a consumer uses
    /// to recognize the same notification re-raised under a new transport event id; at-least-once
    /// redelivery of the *same* bus record is keyed by `event_id`, not by this.
    pub struct NotificationId,
    kind = "notification",
    description = "Identity of one raised user-facing notification. Bare canonical lowercase \
                   hyphenated UUID; not namespaced."
}

uuid_newtype! {
    /// A correlation identity minted by a producer for work not bound to an operation.
    ///
    /// Wire form: bare canonical lowercase hyphenated UUID. The envelope's `correlation_id` slot
    /// is an [`EntityRef`] instead, so `operation:`, `command:` and future kinds fit there without
    /// an envelope major bump; use `CorrelationId::as_entity_ref` to put one of these into it.
    ///
    /// `correlation` is deliberately **not** an [`EntityKind`] variant and not in
    /// `contracts.toml [entity_kinds].known`: a correlation scope is not a Ratatoskr entity, so
    /// widening one lands in the open vocabulary as `EntityKind::Other("correlation")`. A fixture
    /// that wants to carry `correlation:` must add the token to `contracts.toml` first, which is
    /// the governed path that file describes (ADR-0007).
    pub struct CorrelationId,
    kind = "correlation",
    description = "A correlation identity minted by a producer for work not bound to an \
                   operation. Bare canonical lowercase hyphenated UUID; not namespaced."
}
