//! The vault connection events and their vocabulary.

use ratatoskr_event_envelope::EventPayload;
use ratatoskr_identifiers::{NotesConnectionId, wire_string_newtype};

wire_string_newtype! {
    /// The display name of a connected vault, as the owner sees it.
    ///
    /// 1 to 255 characters with no C0 or DEL control character. The pattern carries the
    /// 255-character bound; the length guard is 1020 UTF-8 bytes, the most 255 characters can
    /// take. It is a label the owner chose, never a host path or a vault secret.
    pub struct VaultDisplayName {
        pattern  = r"^[^\x00-\x1f\x7f]{1,255}$",
        max_len  = 1020,
        examples = ["Personal", "Work notes"],
    }
}

/// How a vault reaches `ratatoskr-notes`.
///
/// **Closed on purpose**: a consumer that does not know a channel must stop rather than guess
/// which sync rules apply.
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
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum VaultChannel {
    /// Obsidian Sync, run headless by `ratatoskr-notes` on the host.
    Sync,
    /// The Ratatoskr Obsidian plugin sends the changes.
    Plugin,
    /// The export agent watches the vault folder on a macOS device.
    Cli,
}

/// Payload of `notes.vault.connected.v1`: one vault connection became active.
///
/// A fact: `ratatoskr-notes` publishes it once, when the connection becomes active. The enclosing
/// envelope carries the owner as `tenant_id` (`user:<id>`) and the aggregate
/// `notes_connection:<connection_id>`; the payload does not repeat the owner.
///
/// It names the connection, the channel and the display name, and nothing else about the vault.
/// Every other member is refused.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct VaultConnected {
    /// The connection that became active.
    pub connection_id: NotesConnectionId,

    /// How the vault reaches `ratatoskr-notes`.
    pub channel: VaultChannel,

    /// The display name of the vault.
    pub vault_display_name: VaultDisplayName,
}

impl EventPayload for VaultConnected {
    const EVENT_TYPE: &'static str = "notes.vault.connected.v1";
}

/// Payload of `notes.vault.disconnected.v1`: one vault connection was removed.
///
/// A fact: `ratatoskr-notes` publishes it once, when the owner removes the connection. The
/// enclosing envelope carries the owner as `tenant_id` (`user:<id>`) and the aggregate
/// `notes_connection:<connection_id>`; the payload does not repeat the owner.
///
/// It names the connection, the channel and the display name, and nothing else about the vault.
/// Every other member is refused.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct VaultDisconnected {
    /// The connection that was removed.
    pub connection_id: NotesConnectionId,

    /// How the vault reached `ratatoskr-notes`.
    pub channel: VaultChannel,

    /// The display name of the vault when it was removed.
    pub vault_display_name: VaultDisplayName,
}

impl EventPayload for VaultDisconnected {
    const EVENT_TYPE: &'static str = "notes.vault.disconnected.v1";
}
