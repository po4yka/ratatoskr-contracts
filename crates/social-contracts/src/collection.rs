//! [`SocialFolderMembership`]: membership in a provider-native folder.

use ratatoskr_identifiers::EntityLocalId;

use crate::values::DisplayName;

/// Membership of this source in one provider-native folder (e.g. an X bookmark folder).
///
/// Provider-native only, on purpose: `AGENTS.md` separates native collections from
/// Ratatoskr-side ones, and no current consumer changeset needs the latter — when one does,
/// that is a new field with its own authority story, not a second variant smuggled in here.
///
/// Membership says nothing about [`SavedAuthority`](crate::SavedAuthority): it is populated
/// only where the provider actually exposes folders through a supported channel. An Instagram
/// or Threads explicit capture carries none, and must never be read as native Saved-list
/// membership.
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
pub struct SocialFolderMembership {
    /// The provider's id for the folder, opaque and case-sensitive.
    pub folder_id: EntityLocalId,

    /// The provider-authored folder name as last observed, when the provider exposes one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_name: Option<DisplayName>,
}
