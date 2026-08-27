//! `contracts.toml` — the machine-readable governance metadata — and its validation against
//! reality (rules R1–R9, R12–R14 of the specification; R10 and R11 are the same predicates as
//! lint rules L4 and L1/L7 and live in [`crate::lint`], so they are reported once, not twice).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::check::Finding;
use crate::registry;

/// The whole of `contracts.toml`.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    /// Format version of this file. Only `1` is understood (R1).
    pub metadata_version: u32,
    /// The closed service vocabulary every owner, producer and consumer is drawn from.
    pub services: Services,
    /// The entity kinds fixtures are checked against (R12).
    pub entity_kinds: EntityKinds,
    /// The field-lint vocabulary (§6.5), kept as data so the lint can read it.
    pub lint: LintVocabulary,
    /// Every contract this repository governs.
    #[serde(rename = "contract")]
    pub contracts: Vec<Contract>,
}

/// The closed service vocabulary.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Services {
    /// Every deployable that may own, produce or consume a contract here.
    pub known: Vec<String>,
}

/// The entity kinds this build names.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityKinds {
    /// Kinds the wire type maps to a named variant. The wire stays open; this list is what
    /// fixtures are checked against, so a typo still fails.
    pub known: Vec<String>,
}

/// The lint vocabulary.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LintVocabulary {
    /// Property names that need a justified waiver (L1).
    pub banned_property_names: Vec<String>,
    /// Suffixes that give an integer property an explicit unit (L2).
    pub required_numeric_suffixes: Vec<String>,
    /// The only property names permitted to carry `format: date-time` (L4).
    pub timestamp_property_names: Vec<String>,
    /// Shortest justification a vague-name waiver may carry (L7).
    pub min_waiver_justification_chars: usize,
}

/// One governed contract.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    /// `<context>.<aggregate>` identity, e.g. `core.event_envelope`.
    pub id: String,
    /// Directory family under `schemas/`, e.g. `core`, `platform` or `events`.
    pub family: String,
    /// Contract major version; at least 1.
    pub major_version: u32,
    /// Where the contract is in its life.
    pub lifecycle: Lifecycle,
    /// Distribution class; `SECURITY.md` keeps public and privileged contracts separate.
    pub classification: Classification,
    /// The bounded context accountable for the contract.
    pub owner: String,
    /// Every deployable that emits this contract.
    pub producers: Vec<String>,
    /// Every deployable that reads this contract.
    pub consumers: Vec<String>,
    /// The workspace crate holding the canonical Rust type.
    pub crate_name: String,
    /// Repository-relative path of the file declaring the canonical Rust type.
    pub canonical_path: String,
    /// Repository-relative directory holding this contract's fixtures.
    pub fixtures_dir: String,
    /// One-line human summary; not a wire artifact.
    pub summary: String,
    /// The root types generated from this contract.
    #[serde(rename = "root_type", default)]
    pub root_types: Vec<RootTypeDecl>,
    /// Field-level governance entries.
    #[serde(rename = "field", default)]
    pub fields: Vec<FieldGovernance>,
    /// Waivers for property names the lint bans.
    #[serde(rename = "vague_field_waiver", default)]
    pub vague_field_waivers: Vec<VagueFieldWaiver>,
    /// Event registration. Legal only when `family = "events"` (R9).
    pub event: Option<EventDecl>,
    /// Command registration. Legal only when `family = "commands"` (R10).
    pub command: Option<CommandDecl>,
}

/// One generated root type.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootTypeDecl {
    /// Fully qualified Rust path; must match a key in the compiled registry (R2).
    pub rust_path: String,
    /// Repository-relative output path of the generated artifact (R3).
    pub output: String,
    /// The `$id` written into the generated artifact (R4).
    pub schema_id: String,
    /// What a reader does with a member it does not know.
    pub unknown_policy: UnknownPolicy,
    /// Privacy class of the contract's contents.
    pub privacy: Privacy,
}

/// Field-level governance for one property.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldGovernance {
    /// JSON pointer to the property, relative to the contract root, or a `TypeName#/...` key
    /// when the property belongs to a reusable nested type.
    pub pointer: String,
    /// Which clock or actor the value comes from.
    pub authority: Authority,
    /// Whether the member is always present.
    pub nullability: Nullability,
    /// The unit or canonical form the value is expressed in.
    pub unit: String,
    /// Prose contract for the member; this is what makes a vague name precise.
    pub note: String,
}

/// A waiver for a property name the lint bans.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VagueFieldWaiver {
    /// JSON pointer to the property, relative to the contract root, or a `TypeName#/...` key
    /// when the property belongs to a reusable nested type.
    pub pointer: String,
    /// Why the banned name is nonetheless a precise contract.
    pub justification: String,
}

/// The registration of an event type.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventDecl {
    /// `<bounded_context>.<aggregate>.<action>.v<major>`.
    pub event_type: String,
    /// Rust path of the payload type; must equal the declared root type (R9).
    pub payload_type: String,
}

/// The registration of a command type.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandDecl {
    /// `<bounded_context>.<aggregate>.<action>.v<major>`.
    pub command_type: String,
    /// Rust path of the payload type; must equal the declared root type (R10).
    pub payload_type: String,
}

/// Where a contract is in its life (`DATA_MODEL.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lifecycle {
    /// Drafted, not yet agreed with consumers.
    Proposed,
    /// Agreed, not yet emitted in production.
    Accepted,
    /// In production use.
    Published,
    /// Still emitted, scheduled for removal.
    Deprecated,
    /// No longer emitted.
    Removed,
}

/// Distribution class (`SECURITY.md` separation rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    /// Reachable by an end-user client.
    Public,
    /// Between Ratatoskr deployables only.
    Internal,
    /// Administrative surface.
    Privileged,
}

/// What a reader does with a member it does not know.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownPolicy {
    /// Keep it and re-emit it verbatim.
    Preserve,
    /// Refuse the record.
    Reject,
}

/// Privacy class of a contract's contents.
///
/// There is deliberately **no** `credential` variant: a credential-bearing contract cannot be
/// declared at all, it fails at parse (`SECURITY.md` baseline, test `M-9`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Privacy {
    /// Routing and lifecycle metadata crossing a process boundary.
    BoundaryMetadata,
    /// Content a user authored or imported.
    UserContent,
    /// Identity of a person or tenant.
    Identity,
    /// Audit and provenance records.
    Audit,
}

/// Which clock or actor a value comes from (`AGENTS.md` principle 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Authority {
    /// The producing bounded context asserts it.
    ProducerAsserted,
    /// An external provider authored it.
    ProviderAuthored,
    /// Ratatoskr observed it happening.
    Observed,
    /// Ratatoskr received it from elsewhere.
    Received,
    /// Ratatoskr processed a record at this instant.
    Processed,
    /// Computed from other members.
    Derived,
}

/// Whether a member is always present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Nullability {
    /// Always present.
    Required,
    /// Absent — never `null` — when there is nothing to say.
    OmittedWhenAbsent,
}

impl Metadata {
    /// The conventional location of the metadata file inside a repository root.
    pub const FILE_NAME: &'static str = "contracts.toml";

    /// Parses `contracts.toml` text (R1).
    ///
    /// # Errors
    ///
    /// The TOML parse error text, which `deny_unknown_fields` turns into a precise report of an
    /// unrecognised key.
    pub fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|error| error.to_string())
    }

    /// The contract owning `rust_path`, if any.
    #[must_use]
    pub fn contract_of(&self, rust_path: &str) -> Option<(&Contract, &RootTypeDecl)> {
        self.contracts.iter().find_map(|contract| {
            contract
                .root_types
                .iter()
                .find(|declared| declared.rust_path == rust_path)
                .map(|declared| (contract, declared))
        })
    }
}

/// The kebab-case spelling of a `snake_case` token.
#[must_use]
pub fn kebab(token: &str) -> String {
    token.replace('_', "-")
}

/// Runs metadata rules R1–R10, R12–R14 against the repository.
///
/// `generated` is the in-memory output of [`crate::generate`]; it supplies the set of paths the
/// registry actually produces, which is one half of R2 and R6.
#[must_use]
pub fn validate(
    metadata: &Metadata,
    root: &Path,
    generated: &BTreeMap<PathBuf, String>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    rule_r1(metadata, &mut findings);
    rule_r2(metadata, &mut findings);
    for contract in &metadata.contracts {
        rule_r3_r4(contract, &mut findings);
        rule_r5(contract, root, &mut findings);
        rule_r7(contract, metadata, &mut findings);
        rule_r9(contract, &mut findings);
        rule_r10(contract, &mut findings);
    }
    rule_r6(metadata, generated, &mut findings);
    rule_r8(metadata, root, &mut findings);
    rule_r12_r13(metadata, root, &mut findings);
    findings
}

/// R1 — the format version this build understands. The rest of R1 is `deny_unknown_fields`,
/// enforced by [`Metadata::parse`].
fn rule_r1(metadata: &Metadata, findings: &mut Vec<Finding>) {
    if metadata.metadata_version != 1 {
        findings.push(Finding::Metadata {
            rule: "R1",
            detail: format!(
                "metadata_version is {}; this build understands 1",
                metadata.metadata_version
            ),
        });
    }
}

/// R2 — the declared root types and the compiled registry are the same set, both directions.
fn rule_r2(metadata: &Metadata, findings: &mut Vec<Finding>) {
    let declared: BTreeSet<&str> = metadata
        .contracts
        .iter()
        .flat_map(|contract| contract.root_types.iter())
        .map(|root_type| root_type.rust_path.as_str())
        .collect();
    let compiled: BTreeSet<&str> = registry::root_types()
        .iter()
        .map(|root_type| root_type.rust_path)
        .collect();
    for missing in compiled.difference(&declared) {
        findings.push(Finding::Metadata {
            rule: "R2",
            detail: format!("{missing} is in the generator registry but not in contracts.toml"),
        });
    }
    for missing in declared.difference(&compiled) {
        findings.push(Finding::Metadata {
            rule: "R2",
            detail: format!("{missing} is declared in contracts.toml but not in the registry"),
        });
    }
}

/// R3 and R4 — output path and `$id` follow the one mechanical convention.
fn rule_r3_r4(contract: &Contract, findings: &mut Vec<Finding>) {
    for root_type in &contract.root_types {
        let expected_output = if contract.family == "events" {
            match contract.event.as_ref() {
                Some(event) => format!("schemas/events/{}.schema.json", event.event_type),
                None => continue,
            }
        } else {
            let suffix = contract.id.split_once('.').map_or("", |(_, tail)| tail);
            format!(
                "schemas/json-schema/{}/{}.v{}.schema.json",
                contract.family,
                kebab(suffix),
                contract.major_version
            )
        };
        if root_type.output != expected_output {
            findings.push(Finding::Metadata {
                rule: "R3",
                detail: format!(
                    "{} declares output {} but the convention is {expected_output}",
                    contract.id, root_type.output
                ),
            });
        }
        let expected_id = format!(
            "urn:ratatoskr:contracts:{}:v{}:{}",
            contract.family,
            contract.major_version,
            registry::short_name(&root_type.rust_path)
        );
        if root_type.schema_id != expected_id {
            findings.push(Finding::Metadata {
                rule: "R4",
                detail: format!(
                    "{} declares schema_id {} but the convention is {expected_id}",
                    contract.id, root_type.schema_id
                ),
            });
        }
    }
}

/// R5 — every declared path exists, and `canonical_path` names its root type.
fn rule_r5(contract: &Contract, root: &Path, findings: &mut Vec<Finding>) {
    if !root.join(&contract.canonical_path).is_file() {
        findings.push(Finding::Metadata {
            rule: "R5",
            detail: format!(
                "{}: canonical_path {} does not exist",
                contract.id, contract.canonical_path
            ),
        });
    } else if let Ok(source) = std::fs::read_to_string(root.join(&contract.canonical_path)) {
        for root_type in &contract.root_types {
            let identifier = registry::short_name(&root_type.rust_path);
            if !source.contains(identifier) {
                findings.push(Finding::Metadata {
                    rule: "R5",
                    detail: format!(
                        "{}: canonical_path {} does not mention {identifier}",
                        contract.id, contract.canonical_path
                    ),
                });
            }
        }
    }
    if !root.join(&contract.fixtures_dir).is_dir() {
        findings.push(Finding::Metadata {
            rule: "R5",
            detail: format!(
                "{}: fixtures_dir {} does not exist",
                contract.id, contract.fixtures_dir
            ),
        });
    }
    for root_type in &contract.root_types {
        if !root.join(&root_type.output).is_file() {
            findings.push(Finding::Metadata {
                rule: "R5",
                detail: format!(
                    "{}: output {} does not exist; run `cargo contracts generate`",
                    contract.id, root_type.output
                ),
            });
        }
    }
}

/// R6 — every generated path is claimed by exactly one root type.
///
/// The other half of R6 — a `*.schema.json` under `schemas/**` that no root type claims — is
/// reported as [`Finding::Orphan`] by [`crate::check`], because it is a fact about the tree
/// rather than about the metadata.
fn rule_r6(
    metadata: &Metadata,
    generated: &BTreeMap<PathBuf, String>,
    findings: &mut Vec<Finding>,
) {
    let mut claims: BTreeMap<&str, usize> = BTreeMap::new();
    for contract in &metadata.contracts {
        for root_type in &contract.root_types {
            *claims.entry(root_type.output.as_str()).or_default() += 1;
        }
    }
    for (output, count) in claims {
        if count > 1 {
            findings.push(Finding::Metadata {
                rule: "R6",
                detail: format!("{output} is claimed by {count} root types; exactly one may"),
            });
        }
    }
    // Claims govern the authored `output` column, which names the JSON Schema family. The
    // TypeScript counterparts are not authored anywhere: their paths are derived mechanically
    // from the very same rows (design D1 of add-typescript-generation), so claiming them again
    // would duplicate the naming authority D1 exists to avoid.
    for output in generated
        .keys()
        .filter(|path| path.to_string_lossy().ends_with(crate::SCHEMA_SUFFIX))
    {
        let claimed = metadata.contracts.iter().any(|contract| {
            contract
                .root_types
                .iter()
                .any(|root_type| Path::new(&root_type.output) == output)
        });
        if !claimed {
            findings.push(Finding::Metadata {
                rule: "R6",
                detail: format!(
                    "{} is generated but claimed by no root type",
                    output.display()
                ),
            });
        }
    }
}

/// R7 — owner, producers and consumers are known services, and neither list is empty.
fn rule_r7(contract: &Contract, metadata: &Metadata, findings: &mut Vec<Finding>) {
    let known: BTreeSet<&str> = metadata.services.known.iter().map(String::as_str).collect();
    let mut check_token = |role: &str, token: &str| {
        if !known.contains(token) {
            findings.push(Finding::Metadata {
                rule: "R7",
                detail: format!(
                    "{}: {role} {token:?} is not in [services].known",
                    contract.id
                ),
            });
        }
    };
    check_token("owner", &contract.owner);
    for producer in &contract.producers {
        check_token("producer", producer);
    }
    for consumer in &contract.consumers {
        check_token("consumer", consumer);
    }
    if contract.producers.is_empty() {
        findings.push(Finding::Metadata {
            rule: "R7",
            detail: format!("{}: producers is empty", contract.id),
        });
    }
    if contract.consumers.is_empty() {
        findings.push(Finding::Metadata {
            rule: "R7",
            detail: format!("{}: consumers is empty", contract.id),
        });
    }
}

/// R8 — every `crate_name` is a workspace member.
fn rule_r8(metadata: &Metadata, root: &Path, findings: &mut Vec<Finding>) {
    let members = workspace_member_names(root);
    for contract in &metadata.contracts {
        if !members.contains(contract.crate_name.as_str()) {
            findings.push(Finding::Metadata {
                rule: "R8",
                detail: format!(
                    "{}: crate_name {} is not a workspace member",
                    contract.id, contract.crate_name
                ),
            });
        }
    }
}

/// R9 — event registration is present exactly for the events family and agrees with the type.
fn rule_r9(contract: &Contract, findings: &mut Vec<Finding>) {
    let is_events_family = contract.family == "events";
    let Some(event) = contract.event.as_ref() else {
        if is_events_family {
            findings.push(Finding::Metadata {
                rule: "R9",
                detail: format!(
                    "{}: family is `events` but [contract.event] is absent",
                    contract.id
                ),
            });
        }
        return;
    };
    if !is_events_family {
        findings.push(Finding::Metadata {
            rule: "R9",
            detail: format!(
                "{}: [contract.event] is legal only when family = \"events\"",
                contract.id
            ),
        });
    }
    match ratatoskr_event_envelope::EventType::parse(&event.event_type) {
        Ok(parsed) => {
            if parsed.major() != contract.major_version {
                findings.push(Finding::Metadata {
                    rule: "R9",
                    detail: format!(
                        "{}: event_type major {} disagrees with major_version {}",
                        contract.id,
                        parsed.major(),
                        contract.major_version
                    ),
                });
            }
        }
        Err(error) => findings.push(Finding::Metadata {
            rule: "R9",
            detail: format!("{}: event_type does not parse: {error}", contract.id),
        }),
    }
    let declares_payload = contract
        .root_types
        .iter()
        .any(|root_type| root_type.rust_path == event.payload_type);
    if !declares_payload {
        findings.push(Finding::Metadata {
            rule: "R9",
            detail: format!(
                "{}: payload_type {} is not one of this contract's root types",
                contract.id, event.payload_type
            ),
        });
    }
    match registry::event_payload_types().get(event.payload_type.as_str()) {
        Some(declared) if *declared == event.event_type => {}
        Some(declared) => findings.push(Finding::Metadata {
            rule: "R9",
            detail: format!(
                "{}: event_type {} disagrees with EventPayload::EVENT_TYPE {declared}",
                contract.id, event.event_type
            ),
        }),
        None => findings.push(Finding::Metadata {
            rule: "R9",
            detail: format!(
                "{}: payload_type {} does not implement EventPayload in this build",
                contract.id, event.payload_type
            ),
        }),
    }
}

/// R10 — command registration is present exactly for the commands family and agrees with the
/// payload type, including the major version and `CommandPayload::COMMAND_TYPE`.
fn rule_r10(contract: &Contract, findings: &mut Vec<Finding>) {
    let is_commands_family = contract.family == "commands";
    let Some(command) = contract.command.as_ref() else {
        if is_commands_family {
            findings.push(Finding::Metadata {
                rule: "R10",
                detail: format!(
                    "{}: family is `commands` but [contract.command] is absent",
                    contract.id
                ),
            });
        }
        return;
    };
    if !is_commands_family {
        findings.push(Finding::Metadata {
            rule: "R10",
            detail: format!(
                "{}: [contract.command] is legal only when family = `commands`",
                contract.id
            ),
        });
    }
    match ratatoskr_event_envelope::CommandType::parse(&command.command_type) {
        Ok(parsed) if parsed.major() == contract.major_version => {}
        Ok(parsed) => findings.push(Finding::Metadata {
            rule: "R10",
            detail: format!(
                "{}: command_type major {} disagrees with major_version {}",
                contract.id,
                parsed.major(),
                contract.major_version
            ),
        }),
        Err(error) => findings.push(Finding::Metadata {
            rule: "R10",
            detail: format!("{}: command_type does not parse: {error}", contract.id),
        }),
    }
    let declares_payload = contract
        .root_types
        .iter()
        .any(|root_type| root_type.rust_path == command.payload_type);
    if !declares_payload {
        findings.push(Finding::Metadata {
            rule: "R10",
            detail: format!(
                "{}: payload_type {} is not one of this contract's root types",
                contract.id, command.payload_type
            ),
        });
    }
    match registry::command_payload_types().get(command.payload_type.as_str()) {
        Some(declared) if *declared == command.command_type => {}
        Some(declared) => findings.push(Finding::Metadata {
            rule: "R10",
            detail: format!(
                "{}: command_type {} disagrees with CommandPayload::COMMAND_TYPE {declared}",
                contract.id, command.command_type
            ),
        }),
        None => findings.push(Finding::Metadata {
            rule: "R10",
            detail: format!(
                "{}: payload_type {} does not implement CommandPayload in this build",
                contract.id, command.payload_type
            ),
        }),
    }
}

/// R12 and R13 — the fixture tree is registered, and every entity kind in it is known.
fn rule_r12_r13(metadata: &Metadata, root: &Path, findings: &mut Vec<Finding>) {
    let known_kinds: BTreeSet<&str> = metadata
        .entity_kinds
        .known
        .iter()
        .map(String::as_str)
        .collect();
    let declared_dirs: Vec<&str> = metadata
        .contracts
        .iter()
        .map(|contract| contract.fixtures_dir.as_str())
        .collect();
    let expectations = crate::fixtures::load_expectations(root);
    let mut registered_invalid = BTreeSet::new();

    for absolute in crate::fixtures::walk_json(&root.join("fixtures")) {
        let Ok(relative) = absolute.strip_prefix(root) else {
            continue;
        };
        let relative_text = relative.to_string_lossy().replace('\\', "/");
        let Some(bucket) = declared_dirs
            .iter()
            .find(|dir| relative_text.starts_with(&format!("{dir}/")))
            .and_then(|dir| {
                relative_text
                    .strip_prefix(&format!("{dir}/"))
                    .map(|tail| (*dir, tail.to_owned()))
            })
        else {
            findings.push(Finding::Metadata {
                rule: "R13",
                detail: format!("{relative_text} is not inside any declared fixtures_dir"),
            });
            continue;
        };
        let (dir, tail) = bucket;
        let legal_bucket = tail.starts_with("valid/")
            || tail.starts_with("invalid/")
            || tail.starts_with("compat/old-consumer-new-producer/")
            || tail.starts_with("compat/new-consumer-old-producer/");
        if !legal_bucket {
            findings.push(Finding::Metadata {
                rule: "R13",
                detail: format!(
                    "{relative_text} is not under valid/, invalid/ or a compat/ direction of {dir}"
                ),
            });
        }
        if tail.starts_with("invalid/") {
            let key = crate::fixtures::expectation_key(dir, &tail);
            if expectations.contains_key(&key) {
                registered_invalid.insert(key);
            } else {
                findings.push(Finding::Metadata {
                    rule: "R13",
                    detail: format!(
                        "{relative_text} has no entry in fixtures/invalid-expectations.toml"
                    ),
                });
            }
        }
        for kind in crate::fixtures::entity_kinds_in(&absolute) {
            if !known_kinds.contains(kind.as_str()) {
                findings.push(Finding::Metadata {
                    rule: "R12",
                    detail: format!(
                        "{relative_text} uses entity kind {kind:?}, \
                         which is not in [entity_kinds].known"
                    ),
                });
            }
        }
    }

    for key in expectations.keys() {
        if !registered_invalid.contains(key) {
            findings.push(Finding::Metadata {
                rule: "R13",
                detail: format!(
                    "fixtures/invalid-expectations.toml registers {key}, which has no file"
                ),
            });
        }
    }
}

/// Every `package.name` of every workspace member, read from the root `Cargo.toml`.
fn workspace_member_names(root: &Path) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let Ok(text) = std::fs::read_to_string(root.join("Cargo.toml")) else {
        return names;
    };
    let Ok(manifest) = toml::from_str::<toml::Value>(&text) else {
        return names;
    };
    let Some(members) = manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
    else {
        return names;
    };
    for member in members.iter().filter_map(toml::Value::as_str) {
        let Ok(member_text) = std::fs::read_to_string(root.join(member).join("Cargo.toml")) else {
            continue;
        };
        if let Ok(member_manifest) = toml::from_str::<toml::Value>(&member_text)
            && let Some(name) = member_manifest
                .get("package")
                .and_then(|package| package.get("name"))
                .and_then(toml::Value::as_str)
        {
            names.insert(name.to_owned());
        }
    }
    names
}
