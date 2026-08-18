//! The three validated vocabularies an operation snapshot carries.

use ratatoskr_identifiers::wire_string_newtype;

wire_string_newtype! {
    /// What work the operation performs, e.g. `content.document.extract`.
    /// Present tense: a kind names an activity, not a completed fact — unlike an event action.
    /// Open by construction (a validated string), so there is no unknown-variant problem.
    pub struct OperationKind {
        pattern  = r"^[a-z][a-z0-9_]{0,31}(\.[a-z][a-z0-9_]{0,31}){1,3}$",
        max_len  = 128,
        examples = ["content.document.extract", "social.source.sync"],
    }
}

wire_string_newtype! {
    /// Producer-defined phase inside the current status, e.g. `downloading`.
    /// **Display only.** The stage vocabulary is per `kind` and is not a contract: a client must
    /// not branch on it. Branch on `status`.
    pub struct OperationStage {
        pattern  = r"^[a-z][a-z0-9_]{0,63}$",
        max_len  = 64,
        examples = ["downloading", "normalizing"],
    }
}

wire_string_newtype! {
    /// What an [`OperationResultRef`](crate::OperationResultRef) points at, e.g.
    /// `content.document`.
    pub struct OperationResultKind {
        pattern  = r"^[a-z][a-z0-9_]{0,31}(\.[a-z][a-z0-9_]{0,31}){1,3}$",
        max_len  = 128,
        examples = ["content.document"],
    }
}
