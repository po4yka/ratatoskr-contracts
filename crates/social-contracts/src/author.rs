//! [`SocialAuthor`]: the inline provider identity of whoever authored a source.

use ratatoskr_identifiers::EntityLocalId;

use crate::tokens::Platform;
use crate::values::{AuthorHandle, DisplayName};

/// The author of a social source, denormalized into the record.
///
/// Inline on purpose: one captured event carries everything Knowledge needs to attribute and
/// index the source, with no second lookup against an author record this repository does not
/// define. The author's identity is the provider's (`platform` plus `external_author_id`),
/// never a Ratatoskr user: an author is not an account in anyone's library.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SocialAuthor {
    /// Platform the author account lives on. Same token as the source's own platform.
    pub platform: Platform,

    /// The provider's id for the author account, opaque and case-sensitive.
    pub external_author_id: EntityLocalId,

    /// Bare screen name without the `@` presentation prefix, when the provider exposes one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle: Option<AuthorHandle>,

    /// Provider-authored display name, when one exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<DisplayName>,
}
