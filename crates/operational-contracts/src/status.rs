//! Public status response types.

use ratatoskr_identifiers::WireTimestamp;

/// Overall public availability of Ratatoskr.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PublicStatusState {
    /// All public component observations are current and healthy.
    Operational,
    /// Ratatoskr can answer but one or more component observations are impaired.
    Degraded,
    /// A required component cannot serve work.
    Unavailable,
}

/// A stable public component group that reveals no private topology.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PublicComponentId {
    /// Public Edge API readiness.
    Api,
    /// Durable storage readiness.
    Storage,
    /// Command delivery readiness.
    CommandDelivery,
    /// Aggregate readiness of connected product services.
    ConnectedServices,
}

/// Availability of one public component group.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PublicComponentState {
    /// The latest observation is current and healthy.
    Operational,
    /// The component is impaired but some useful operation remains.
    Degraded,
    /// The component cannot serve its required work.
    Unavailable,
    /// No successful observation exists.
    Unknown,
}

/// Sanitized observation of one stable public component group.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PublicStatusComponent {
    /// Stable public component identifier.
    pub id: PublicComponentId,
    /// Latest contracted component state.
    pub state: PublicComponentState,
    /// Latest successful observation, absent when never observed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<WireTimestamp>,
    /// Whether the last successful observation is older than the current failed refresh.
    pub stale: bool,
}

/// Anonymous sanitized public status response.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PublicStatusDocument {
    /// Instant this projection was produced by Platform.
    pub generated_at: WireTimestamp,
    /// Overall public availability.
    pub state: PublicStatusState,
    /// Stable component observations in contract order.
    #[schemars(length(equal = 4))]
    pub components: Vec<PublicStatusComponent>,
}

impl PublicStatusDocument {
    /// Fixed public component order. Order is contract-significant for deterministic rendering.
    pub const COMPONENT_ORDER: [PublicComponentId; 4] = [
        PublicComponentId::Api,
        PublicComponentId::Storage,
        PublicComponentId::CommandDelivery,
        PublicComponentId::ConnectedServices,
    ];

    /// Creates and validates a public status projection.
    ///
    /// # Errors
    ///
    /// Returns [`StatusContractError`] when component order, freshness, or the aggregate state is
    /// inconsistent with the component facts.
    pub fn new(
        generated_at: WireTimestamp,
        state: PublicStatusState,
        components: Vec<PublicStatusComponent>,
    ) -> Result<Self, StatusContractError> {
        let document = Self {
            generated_at,
            state,
            components,
        };
        document.validate()?;
        Ok(document)
    }

    /// Re-checks ordering, freshness, and aggregate-state invariants.
    ///
    /// # Errors
    ///
    /// Returns [`StatusContractError`] for any inconsistent public projection.
    pub fn validate(&self) -> Result<(), StatusContractError> {
        if self.components.len() != Self::COMPONENT_ORDER.len()
            || !self
                .components
                .iter()
                .zip(Self::COMPONENT_ORDER)
                .all(|(component, expected)| component.id == expected)
        {
            return Err(StatusContractError::ComponentOrder);
        }

        for component in &self.components {
            if component.state == PublicComponentState::Operational && component.stale {
                return Err(StatusContractError::OperationalIsStale { id: component.id });
            }
            if component.state == PublicComponentState::Unknown && component.observed_at.is_some() {
                return Err(StatusContractError::UnknownWasObserved { id: component.id });
            }
        }

        let expected = if self.components.iter().any(|component| {
            matches!(
                component.id,
                PublicComponentId::Api | PublicComponentId::Storage
            ) && component.state == PublicComponentState::Unavailable
        }) {
            PublicStatusState::Unavailable
        } else if self.components.iter().all(|component| {
            component.state == PublicComponentState::Operational && !component.stale
        }) {
            PublicStatusState::Operational
        } else {
            PublicStatusState::Degraded
        };

        if self.state != expected {
            return Err(StatusContractError::AggregateMismatch {
                declared: self.state,
                expected,
            });
        }
        Ok(())
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicStatusDocumentWire {
    generated_at: WireTimestamp,
    state: PublicStatusState,
    components: Vec<PublicStatusComponent>,
}

impl<'de> serde::Deserialize<'de> for PublicStatusDocument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = PublicStatusDocumentWire::deserialize(deserializer)?;
        Self::new(wire.generated_at, wire.state, wire.components).map_err(serde::de::Error::custom)
    }
}

/// Invalid status-document invariant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum StatusContractError {
    /// Component identifiers are absent, duplicated, or out of canonical order.
    #[error("public status components must contain the four canonical identifiers in order")]
    ComponentOrder,
    /// A stale observation was labelled operational.
    #[error("operational component {id:?} cannot be stale")]
    OperationalIsStale {
        /// Component carrying the contradictory facts.
        id: PublicComponentId,
    },
    /// An unknown component claimed a successful observation instant.
    #[error("unknown component {id:?} cannot have an observation time")]
    UnknownWasObserved {
        /// Component carrying the contradictory facts.
        id: PublicComponentId,
    },
    /// Overall state did not match the component facts.
    #[error("status aggregate {declared:?} does not match expected {expected:?}")]
    AggregateMismatch {
        /// State carried by the document.
        declared: PublicStatusState,
        /// State computed from component facts.
        expected: PublicStatusState,
    },
}
