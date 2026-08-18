//! Qualified, self-describing wire references: [`EntityKind`], [`EntityLocalId`], [`EntityRef`]
//! and the closed-kind owner reference [`TenantRef`].

use core::convert::TryFrom;
use core::fmt;
use core::str::FromStr;

use crate::error::IdentifierError;
use crate::uuid_ids::{UserId, canonical_uuid};
use crate::wire_string_newtype;

wire_string_newtype! {
    /// The identity half of a qualified reference. Opaque: a UUID, a provider's numeric string,
    /// or a provider slug. May contain `:`, because the kind separator is the FIRST colon only.
    pub struct EntityLocalId {
        pattern  = r"^[A-Za-z0-9][A-Za-z0-9._~:@+-]{0,255}$",
        max_len  = 256,
        examples = ["018f0000-0000-7000-8000-000000000002", "123"],
    }
}

impl EntityLocalId {
    /// Infallible construction from a UUID. `uuid::Uuid`'s `Display` is the canonical lowercase
    /// hyphenated form, which always satisfies `EntityLocalId::PATTERN`.
    pub(crate) fn from_uuid(value: uuid::Uuid) -> Self {
        Self(value.to_string())
    }
}

/// The namespace half of a qualified wire reference.
///
/// **Open on purpose.** An unrecognised kind is preserved verbatim in [`EntityKind::Other`], so a
/// bounded context added in a later milestone (`x-post`, `social_source`, `repository`) does not
/// break a consumer built today: an unknown pointer kind is still routable, ackable and loggable.
/// `DOMAIN.md` invariant 6, "preserved" branch. Typos are caught out of band by
/// `contracts.toml [entity_kinds].known`, which every fixture is checked against.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EntityKind {
    /// A Ratatoskr end user. Wire token `user`.
    User,
    /// A long-running operation. Wire token `operation`.
    Operation,
    /// A domain event. Wire token `event`.
    Event,
    /// A normalized document. Wire token `document`.
    Document,
    /// Stored bytes addressed by content hash. Wire token `blob`.
    Blob,
    /// A kind this build does not know, preserved exactly as received.
    Other(String),
}

impl EntityKind {
    /// Token grammar. Hyphens are permitted because `README.md` shows the kind `x-post`.
    pub const PATTERN: &'static str = r"^[a-z][a-z0-9_-]{0,31}$";

    /// Contract maximum token length in UTF-8 bytes. `PATTERN` also bounds it.
    pub const MAX_LEN: usize = 32;

    /// The tokens this build maps to a named variant, sorted for stable review diffs.
    pub const KNOWN: &'static [&'static str] = &["blob", "document", "event", "operation", "user"];

    /// The published `PATTERN`, compiled once.
    #[allow(
        clippy::expect_used,
        reason = "PATTERN is a compile-time contract constant; a build whose pattern does not \
                  compile is broken before it reaches a wire"
    )]
    fn compiled_pattern() -> &'static regex::Regex {
        static COMPILED: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
            regex::Regex::new(EntityKind::PATTERN)
                .expect("contract PATTERN must be a valid regular expression")
        });
        &COMPILED
    }

    /// Parses a kind token, mapping an unrecognised one to [`EntityKind::Other`].
    ///
    /// # Errors
    ///
    /// [`IdentifierError::PatternMismatch`] when the token grammar is violated.
    pub fn parse(raw: &str) -> Result<Self, IdentifierError> {
        if !Self::compiled_pattern().is_match(raw) {
            return Err(IdentifierError::PatternMismatch {
                type_name: "EntityKind",
                pattern: Self::PATTERN,
                input: raw.to_owned(),
            });
        }
        Ok(Self::from_token(raw))
    }

    /// Maps a token already known to satisfy `PATTERN` onto a variant.
    pub(crate) fn from_token(token: &str) -> Self {
        match token {
            "blob" => Self::Blob,
            "document" => Self::Document,
            "event" => Self::Event,
            "operation" => Self::Operation,
            "user" => Self::User,
            other => Self::Other(other.to_owned()),
        }
    }

    /// The wire token for this kind.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::User => "user",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Document => "document",
            Self::Blob => "blob",
            Self::Other(token) => token,
        }
    }

    /// `true` when this build understands the kind.
    #[must_use]
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Other(_))
    }
}

impl fmt::Display for EntityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for EntityKind {
    type Err = IdentifierError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse(raw)
    }
}

/// A namespaced, polymorphic wire reference, serialized as `"<kind>:<local_id>"`.
///
/// This is the type behind the `aggregate_id`, `correlation_id` and `causation_id` strings in
/// `ARCHITECTURE.md` S5.2 and behind `README.md`'s `"aggregate_id": "x-post:123"`.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub struct EntityRef {
    kind: EntityKind,
    local_id: EntityLocalId,
}

impl EntityRef {
    /// The published JSON Schema `pattern`: `EntityKind::PATTERN`, a colon, then
    /// `EntityLocalId::PATTERN`.
    pub const PATTERN: &'static str =
        r"^[a-z][a-z0-9_-]{0,31}:[A-Za-z0-9][A-Za-z0-9._~:@+-]{0,255}$";

    /// Contract maximum length in UTF-8 bytes: 32 kind + 1 separator + 256 local id.
    pub const MAX_LEN: usize = 289;

    /// Joins an already-validated kind and local identity.
    #[must_use]
    pub fn new(kind: EntityKind, local_id: EntityLocalId) -> Self {
        Self { kind, local_id }
    }

    /// Splits on the **first** colon; the local part may contain further colons.
    ///
    /// # Errors
    ///
    /// [`IdentifierError::MissingKindSeparator`] when there is no colon,
    /// [`IdentifierError::PatternMismatch`] when either half violates its grammar,
    /// [`IdentifierError::Empty`] or [`IdentifierError::TooLong`] for a local identity outside
    /// its bounds.
    pub fn parse(raw: &str) -> Result<Self, IdentifierError> {
        let Some((kind, local_id)) = raw.split_once(':') else {
            return Err(IdentifierError::MissingKindSeparator {
                input: raw.to_owned(),
            });
        };
        Ok(Self {
            kind: EntityKind::parse(kind)?,
            local_id: EntityLocalId::parse(local_id)?,
        })
    }

    /// The namespace half.
    #[must_use]
    pub fn kind(&self) -> &EntityKind {
        &self.kind
    }

    /// The identity half.
    #[must_use]
    pub fn local_id(&self) -> &EntityLocalId {
        &self.local_id
    }

    /// `Some` when the local part is a canonical lowercase hyphenated UUID; `None` for
    /// provider-supplied identities such as `x-post:123`.
    #[must_use]
    pub fn as_uuid(&self) -> Option<uuid::Uuid> {
        canonical_uuid(self.local_id.as_str())
    }

    /// The wire rendering, `"<kind>:<local_id>"`.
    #[must_use]
    pub fn to_wire(&self) -> String {
        let mut out =
            String::with_capacity(self.kind.as_str().len() + 1 + self.local_id.as_str().len());
        out.push_str(self.kind.as_str());
        out.push(':');
        out.push_str(self.local_id.as_str());
        out
    }
}

impl fmt::Display for EntityRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.kind.as_str(), self.local_id.as_str())
    }
}

impl FromStr for EntityRef {
    type Err = IdentifierError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse(raw)
    }
}

impl TryFrom<String> for EntityRef {
    type Error = IdentifierError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::parse(&raw)
    }
}

impl From<EntityRef> for String {
    fn from(value: EntityRef) -> Self {
        value.to_wire()
    }
}

impl schemars::JsonSchema for EntityRef {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("EntityRef")
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::EntityRef"))
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "title": "EntityRef",
            "description": "A namespaced, polymorphic wire reference, serialized as \
                            `<kind>:<local_id>`. The kind vocabulary is open: an unrecognised \
                            kind is preserved verbatim. The local identity is opaque and need \
                            not be a UUID.",
            "pattern": Self::PATTERN,
            "maxLength": Self::MAX_LEN,
            "examples": ["document:018f0000-0000-7000-8000-000000000002", "x-post:123"],
        })
    }
}

/// The owner of the data a record concerns. Wire form `user:<uuid>`.
///
/// **The kind set is closed**, unlike every other reference in this crate. Tenancy governs
/// authorization and data separation; a consumer that does not understand the owner kind must not
/// process the record at all (`SECURITY.md`: security review is required for identity fields).
/// `DOMAIN.md` invariant 6, "rejected explicitly" branch.
///
/// Widening this to organisation tenants is a `pattern` relaxation and therefore backward
/// compatible on the wire, but the classifier reports every `pattern` change as breaking so the
/// change is forced through human review. Adding a *variant* here is a new major envelope version.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub struct TenantRef(UserId);

impl TenantRef {
    /// The published JSON Schema `pattern`. The literal `user:` prefix is the closed kind set.
    pub const PATTERN: &'static str =
        r"^user:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

    /// The only constructor: tenancy is a user today.
    #[must_use]
    pub const fn of_user(id: UserId) -> Self {
        Self(id)
    }

    /// Parses the wire form `user:<uuid>`.
    ///
    /// # Errors
    ///
    /// [`IdentifierError::MissingKindSeparator`] when there is no colon,
    /// [`IdentifierError::KindMismatch`] for any kind other than `user`,
    /// [`IdentifierError::NotAUuid`] for a non-UUID local part.
    pub fn parse(raw: &str) -> Result<Self, IdentifierError> {
        let Some((kind, local_id)) = raw.split_once(':') else {
            return Err(IdentifierError::MissingKindSeparator {
                input: raw.to_owned(),
            });
        };
        if kind != "user" {
            return Err(IdentifierError::KindMismatch {
                expected: "user",
                actual: kind.to_owned(),
            });
        }
        let Some(value) = canonical_uuid(local_id) else {
            return Err(IdentifierError::NotAUuid {
                local_id: local_id.to_owned(),
            });
        };
        Ok(Self(UserId(value)))
    }

    /// Infallible: `user` is the only permitted kind. This is the concrete payoff of the closed
    /// set — no consumer writes an "if the tenant is a user" branch with no else arm.
    #[must_use]
    pub const fn user_id(self) -> UserId {
        self.0
    }

    /// The same identity widened to the open reference vocabulary.
    #[must_use]
    pub fn as_entity_ref(self) -> EntityRef {
        EntityRef::new(EntityKind::User, EntityLocalId::from_uuid(self.user_id().0))
    }
}

impl fmt::Display for TenantRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "user:{}", self.user_id().0)
    }
}

impl FromStr for TenantRef {
    type Err = IdentifierError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse(raw)
    }
}

impl TryFrom<String> for TenantRef {
    type Error = IdentifierError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::parse(&raw)
    }
}

impl From<TenantRef> for String {
    fn from(value: TenantRef) -> Self {
        value.to_string()
    }
}

impl schemars::JsonSchema for TenantRef {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("TenantRef")
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::TenantRef"))
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "title": "TenantRef",
            "description": "The owner of the data a record concerns, wire form `user:<uuid>`. \
                            The kind set is closed: a consumer that cannot understand the owner \
                            kind must reject the record rather than process it.",
            "pattern": Self::PATTERN,
            "maxLength": 41,
            "examples": ["user:018f0000-0000-7000-8000-000000000005"],
        })
    }
}
