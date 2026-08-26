//! The four closed vocabularies a snapshot carries: how a source arrived, what its saved-state
//! claim is worth, whether the capture is whole, and what upstream looks like now.

/// How the source entered the user's library.
///
/// **Closed on purpose.** An unknown value is rejected at parse rather than guessed at: the
/// method determines retention, re-acquisition and provenance handling, and silently filing an
/// unrecognized method under a default is how provenance rots. Adding a variant is an additive
/// wire change that consumers adopt by upgrading.
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
pub enum AcquisitionMethod {
    /// Read from the platform's supported, authenticated API.
    OfficialApi,
    /// The user pushed it in through an OS or browser share target.
    ShareExtension,
    /// The user pushed it in through a browser extension.
    BrowserExtension,
    /// Resolved from the platform's public, unauthenticated metadata surface (oEmbed-style).
    PublicResolution,
    /// Parsed out of a provider data export the user supplied.
    DataExport,
    /// Carried over from the retired monolith's stored state.
    LegacyImport,
}

/// What the saved-state claim in this snapshot is worth.
///
/// **Closed on purpose.** `AGENTS.md`: never model Instagram or Threads explicit capture as
/// authoritative membership in the provider's native Saved list. An unknown authority must stop
/// processing — a consumer that guesses treats a maybe-saved source as saved.
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
pub enum SavedAuthority {
    /// The platform itself exposes this saved state (e.g. X bookmarks through the supported
    /// API). The strongest claim; folder membership may be provider-authoritative too.
    AuthoritativePlatformState,
    /// A user action inside Ratatoskr captured the source; the provider's own state is unknown.
    ExplicitUserCapture,
    /// A provider data export shows the item was saved at some point, without live authority.
    ExportObservation,
    /// Migrated from the retired monolith; authority is as good as that record was.
    LegacyObservation,
}

/// Whether this capture obtained everything the producer knows how to capture.
///
/// **Closed on purpose**: a consumer must not guess whether a capture it cannot classify is
/// safe to treat as whole. `partial` requires at least one warning naming what is missing
/// (enforced where the snapshot is parsed).
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
pub enum CaptureCompleteness {
    /// Everything the producer knows how to capture was captured. May still carry warnings for
    /// problems that did not reduce completeness.
    Complete,
    /// Something known is missing; the warnings say what.
    Partial,
}

/// The observed upstream availability of the source at `captured_at`.
///
/// **Closed on purpose**: availability drives conservative retention and re-fetch decisions,
/// and an unrecognized state must stop processing instead of being read as "fine". A source
/// observed deleted upstream keeps whatever was captured before it went away.
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
pub enum UpstreamAvailability {
    /// The source resolved normally when last observed.
    Available,
    /// The source failed to resolve, but deletion is not proven (rate limits, embed blocks).
    Unavailable,
    /// The provider stated or implied the source no longer exists.
    DeletedUpstream,
}

/// Why a user's library stopped holding a source (`social.source.removed.v1`).
///
/// **Closed on purpose.** A removal fact says the *library* let go; it never claims anything
/// about upstream state, which `UpstreamAvailability` owns. An unknown reason must stop
/// processing instead of being read as an acceptable deletion.
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
pub enum RemovalReason {
    /// The user explicitly asked Ratatoskr to forget this source.
    UserRequested,
    /// A retention policy expired the record; no user action was involved.
    RetentionPolicy,
}
