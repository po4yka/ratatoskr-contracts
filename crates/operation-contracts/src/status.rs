//! The closed operation lifecycle enum.

/// The public lifecycle state of a long-running operation (`ARCHITECTURE.md` S5.4).
///
/// **Closed.** An unrecognised value is rejected at deserialization with an error naming the
/// legal variants (`DOMAIN.md` invariant 6, "rejected explicitly" branch; `ARCHITECTURE.md` S9.2
/// permits enum additions only "when consumers preserve **or safely reject** unknown values").
/// A client that guesses at an unknown lifecycle state reports unfinished work as finished, which
/// is the failure this enum exists to prevent. Adding a state is therefore a new major version.
///
/// `#[non_exhaustive]` so a future addition is not source-breaking for downstream Rust.
///
/// Progress is monotonic in lifecycle semantics (S5.4). That is a producer obligation: it is
/// unenforceable from a single snapshot, and this repository publishes no transition table
/// because a transition table is a business workflow (`AGENTS.md` hard boundaries).
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
pub enum OperationStatus {
    /// The request was received and is durably recorded. No work has started.
    Accepted,
    /// The work is scheduled and waiting for capacity.
    Queued,
    /// The work is executing.
    Running,
    /// Terminal. Every requested effect was produced.
    Succeeded,
    /// Terminal. Some requested effects were produced; `warnings` explains which were not.
    PartiallySucceeded,
    /// Terminal. No usable effect was produced; `errors` is non-empty.
    Failed,
    /// Terminal. Stopped on request before completion.
    Cancelled,
}

impl OperationStatus {
    /// Every documented status, in lifecycle order. The array length is the documented count, so
    /// adding a variant without updating this array does not compile.
    pub const ALL: [Self; 7] = [
        Self::Accepted,
        Self::Queued,
        Self::Running,
        Self::Succeeded,
        Self::PartiallySucceeded,
        Self::Failed,
        Self::Cancelled,
    ];

    /// The wire token, spelled exactly as `ARCHITECTURE.md` S5.4 spells it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::PartiallySucceeded => "partially_succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    /// `true` for `Succeeded`, `PartiallySucceeded`, `Failed`, `Cancelled`. A terminal status is
    /// never left (`ARCHITECTURE.md` S5.4: "Progress is monotonic in lifecycle semantics").
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::PartiallySucceeded | Self::Failed | Self::Cancelled
        )
    }
}

impl core::fmt::Display for OperationStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}
