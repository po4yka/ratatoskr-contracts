//! Projection of normalized JSON Schemas onto self-contained TypeScript declarations.
//!
//! The emitter consumes the same normalized schema values the JSON renderer emits — `$id`
//! included — and produces one complete `.ts` source per contract (design decisions D1–D4 in
//! `openspec/changes/add-typescript-generation/design.md`).
//!
//! # Fail-closed by construction
//!
//! Every schema node is dispatched on the constructs it actually carries, and every node is
//! then *audited*: any keyword that neither the D3 mapping nor the inert annotation list
//! accounts for — `patternProperties`, `if`, string-typed schemas that are not primitives,
//! and so on — aborts generation naming the offending construct and the contract's `$id`
//! ([`GenError::UnrepresentableConstruct`]). The generator never emits an unsound
//! approximation. Annotation and constraint keywords (`description`, `format`, `pattern`,
//! `examples`, …) constrain validity, never shape, so they are inert: they surface as `JSDoc`
//! or are ignored.
//!
//! # Determinism
//!
//! Objects iterate in key order (`serde_json` maps are `BTreeMap` here), unions keep document
//! order with `null` moved last, and no clock or environment value is ever consulted — the
//! same discipline the JSON family follows. Writes into the output buffer are formatted
//! pushes whose failure mode does not exist for a `String`, so they discard the impossible
//! [`std::fmt::Error`] instead of threading it through every signature.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde_json::{Map, Value};

use crate::GenError;

/// The directory every generated TypeScript artifact lives under, mirroring [`crate::SCHEMAS_DIR`].
pub const TYPESCRIPT_DIR: &str = "generated/typescript";

/// Keywords that carry no shape: annotations and validation constraints. Their presence is
/// never an approximation hazard, so the audit skips them.
const INERT_KEYWORDS: &[&str] = &[
    "$schema",
    "$id",
    "$defs",
    "title",
    "description",
    "format",
    "default",
    "examples",
    "pattern",
    "maxLength",
    "minLength",
    "minimum",
    "maximum",
    "exclusiveMinimum",
    "exclusiveMaximum",
    "multipleOf",
    "minItems",
    "maxItems",
    "uniqueItems",
    "x-ratatoskr-unknown-policy",
    "x-ratatoskr-provenance",
];

/// The keywords a node consumes when it is projected as a named object declaration.
const OBJECT_SHAPE_KEYWORDS: &[&str] = &["type", "properties", "required", "additionalProperties"];

/// The projection of a JSON value that is explicitly "anything except an object": the exact
/// complement within the JSON value universe, which has exactly six kinds.
const NON_OBJECT_JSON: &str = "string | number | boolean | null | unknown[]";

/// Derives the TypeScript output path from a JSON Schema output path (design decision D1):
/// the leading `schemas/` segment becomes [`TYPESCRIPT_DIR`] and the trailing
/// [`crate::SCHEMA_SUFFIX`] becomes `.ts`.
///
/// Returns `None` for outputs that do not follow the convention; metadata rule R3 reports
/// those as authored-data problems, so the generator treats them as undeclared rather than
/// guessing a path.
#[must_use]
pub fn typescript_output_path(schema_output: &str) -> Option<String> {
    let stem = schema_output
        .strip_prefix("schemas/")?
        .strip_suffix(crate::SCHEMA_SUFFIX)?;
    Some(format!("{TYPESCRIPT_DIR}/{stem}.ts"))
}

/// Projects one normalized JSON Schema onto its TypeScript source text.
///
/// The input is the fully normalized schema value exactly as [`crate::render`] receives it,
/// `$id` included; the output is the complete `.ts` file body: the root type export named
/// after the final schema-id segment, followed by every `$defs` member exported in sorted
/// order (D4). The leading provenance block comment is rendered around this body when the
/// schema carries one.
///
/// # Errors
///
/// Returns [`GenError::UnrepresentableConstruct`] when the schema uses a construct outside the
/// supported subset, naming the offending schema identifier and construct.
pub fn emit_typescript(schema_id: &str, schema: &Value) -> Result<String, GenError> {
    let mut declarations = String::new();
    write_export(&mut declarations, root_name(schema_id), schema, schema_id)?;

    if let Some(definitions) = schema.get("$defs").and_then(Value::as_object) {
        // Key order is the map's sort order, so definition exports are sorted by construction.
        for (name, definition) in definitions {
            declarations.push('\n');
            write_export(&mut declarations, name, definition, schema_id)?;
        }
    }

    // The provenance block rides in front of the body as a leading comment (design D5). Its
    // `source_digest` member is *re-stamped*: the JSON family digests its own rendered bytes,
    // while this family digests the TypeScript body, so each artifact carries the digest of
    // the bytes it was actually emitted with.
    let header = match schema
        .get(crate::provenance::PROVENANCE_KEY)
        .and_then(Value::as_object)
    {
        Some(block) => {
            let digest = crate::provenance::source_digest(&declarations);
            render_header_block(block, &digest)
        }
        None => String::new(),
    };
    let mut out = header;
    out.push_str(&declarations);
    Ok(out)
}

/// Renders the nine-member provenance block as a leading `/* ... */` comment: the
/// generated-file marker first, then the remaining eight members in their stored (sorted)
/// order, one `key: value` per line.
fn render_header_block(block: &Map<String, Value>, body_digest: &str) -> String {
    let spelled = |value: &Value| match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    };
    let mut header = String::from("/*\n");
    if let Some(marker) = block.get("generated") {
        let _ = writeln!(header, " * generated: {}", spelled(marker));
    }
    let _ = writeln!(header, " *");
    for (key, value) in block {
        if key == "generated" {
            continue;
        }
        let value_text = if key == "source_digest" {
            body_digest.to_owned()
        } else {
            spelled(value)
        };
        let _ = writeln!(header, " * {key}: {value_text}");
    }
    header.push_str(" */\n");
    header
}

/// The final `:`-segment of a schema identifier: the exported root type's name.
fn root_name(schema_id: &str) -> &str {
    schema_id.rsplit(':').next().unwrap_or(schema_id)
}

/// Splits a `.ts` artifact into its leading provenance block comment and the header-less body.
///
/// Returns `None` when the artifact does not start with a `/* ... */` block, which for a
/// generated file means it carries no provenance at all.
#[must_use]
pub fn header_and_body(file: &str) -> Option<(&str, &str)> {
    let rest = file.strip_prefix("/*")?;
    let close = rest.find("*/")?;
    let header_len = 2 + close + 2;
    let header = file.get(..header_len)?;
    let remainder = file.get(header_len..)?;
    // The emitter writes exactly one newline between the closing delimiter and the body, so
    // stripping it back recovers the bytes the digest was taken over.
    let body = remainder.strip_prefix('\n').unwrap_or(remainder);
    Some((header, body))
}

/// Recomputes the `source_digest` a TypeScript artifact's own body implies: SHA-256 over the
/// header-less body, exactly the inverse of what the emitter recorded.
///
/// # Errors
///
/// Returns an explanation when the artifact carries no parseable provenance header.
pub fn recompute_digest(file: &str) -> Result<String, String> {
    let (_, body) = header_and_body(file)
        .ok_or_else(|| "the artifact carries no provenance header".to_owned())?;
    Ok(crate::provenance::source_digest(body))
}

/// The embedded `source_digest` of a TypeScript artifact, read from its leading block comment.
#[must_use]
pub fn embedded_digest(file: &str) -> Option<String> {
    let (header, _) = header_and_body(file)?;
    header.lines().find_map(|line| {
        let content = line.trim().strip_prefix("* ")?;
        let (key, value) = content.split_once(": ")?;
        (key == "source_digest").then(|| value.to_owned())
    })
}

/// What one compiler spawn produced, abstracted just past [`std::process::Output`] so tests can
/// synthesize outcomes without constructing an `ExitStatus`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnOutcome {
    /// Whether the child exited successfully.
    pub success: bool,
    /// Combined standard output and error, as the compiler spelled it.
    pub output: String,
}

/// How a `check-typescript` run ended (design D7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileVerdict {
    /// The scratch project compiled with zero errors.
    Compiled,
    /// The compiler ran and reported diagnostics; the payload is its own output.
    Diagnostics(String),
    /// No compiler could be spawned; the payload explains the override and install options.
    Unavailable(String),
}

/// Compiles the current TypeScript outputs under a throwaway strict-mode project.
///
/// The outputs of [`crate::generate`] are materialized into a temporary directory holding a
/// minimal strict `tsconfig.json` (`strict`, `noEmit`), then `tsc` is spawned over the project
/// through the injected runner. Resolution order: the caller-supplied environment override
/// (the CLI passes `CONTRACTSC_TSC`) first, then `npx --no-install tsc`. The first candidate
/// that *spawns* decides the verdict; a candidate that cannot spawn falls through to the next,
/// and exhausting both yields actionable guidance rather than a raw npx error (design D7).
///
/// # Errors
///
/// Returns [`GenError`] when generation itself fails before any compilation is attempted, or
/// when the scratch project cannot be written.
pub fn check_typescript_with(
    metadata: &crate::Metadata,
    generator_version: &str,
    env_override: Option<&str>,
    spawn: impl Fn(&std::path::Path, &str, &[String]) -> std::io::Result<SpawnOutcome>,
) -> Result<CompileVerdict, GenError> {
    let generated = crate::generate(metadata, generator_version)?;
    let project = ScratchProject::materialize(&generated)?;

    let mut candidates: Vec<(String, Vec<String>)> = Vec::new();
    if let Some(override_program) = env_override {
        candidates.push((
            override_program.to_owned(),
            vec!["--noEmit".to_owned(), "-p".to_owned(), ".".to_owned()],
        ));
    }
    candidates.push((
        "npx".to_owned(),
        vec![
            "--no-install".to_owned(),
            "tsc".to_owned(),
            "--noEmit".to_owned(),
            "-p".to_owned(),
            ".".to_owned(),
        ],
    ));

    let mut failures = String::new();
    for (program, args) in &candidates {
        match spawn(&project.path, program, args) {
            Ok(outcome) => {
                // A spawned-but-unresolved fallback is still *unresolved*: `npx
                // --no-install tsc` without TypeScript installed cancels with a package
                // resolution error, which must produce guidance, not a compiler diagnostic.
                if !outcome.success && is_resolution_failure(program, &outcome.output) {
                    let _ = writeln!(failures, "  {program}: {}", outcome.output.trim());
                    continue;
                }
                return Ok(if outcome.success {
                    CompileVerdict::Compiled
                } else {
                    CompileVerdict::Diagnostics(outcome.output)
                });
            }
            Err(error) => {
                let _ = writeln!(failures, "  {program}: {error}");
            }
        }
    }
    Ok(CompileVerdict::Unavailable(format!(
        "No TypeScript compiler could be spawned:\n{failures}Install one with \
         `npm install -g typescript` (or locally, so `npx --no-install tsc` finds it), or set \
         the CONTRACTSC_TSC environment variable to an explicit `tsc` binary."
    )))
}

/// `true` when the fallback runner ran but could not *resolve* a compiler: npm's cancellation
/// for a missing package, or a shell's not-found complaint. Real TypeScript diagnostics name
/// error codes (`error TS…`), never these phrases.
fn is_resolution_failure(program: &str, output: &str) -> bool {
    if program != "npx" {
        return false;
    }
    output.contains("canceled due to missing packages")
        || output.contains("could not determine executable")
        || output.contains("command not found")
}

/// A throwaway directory holding one generated run, removed when the guard drops.
struct ScratchProject {
    /// Root of the scratch project: `tsconfig.json` plus every generated `.ts`.
    path: std::path::PathBuf,
}

impl ScratchProject {
    /// Writes `tsconfig.json` and every TypeScript artifact under a fresh temporary root.
    ///
    /// The root is unique per call — process id plus a monotonically increasing counter — so
    /// concurrent invocations inside one test binary neither collide nor delete each other's
    /// project mid-run.
    fn materialize(generated: &BTreeMap<std::path::PathBuf, String>) -> Result<Self, GenError> {
        static NEXT_PROJECT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
        let ordinal = NEXT_PROJECT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ratatoskr-contractsc-tscheck-{}-{ordinal}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).map_err(|error| GenError::Io {
            path: path.clone(),
            detail: error.to_string(),
        })?;
        std::fs::write(path.join("tsconfig.json"), STRICT_TSCONFIG).map_err(|error| {
            GenError::Io {
                path: path.join("tsconfig.json"),
                detail: error.to_string(),
            }
        })?;
        for (relative, body) in generated {
            if !relative.to_string_lossy().ends_with(".ts") {
                continue;
            }
            let absolute = path.join(relative);
            if let Some(parent) = absolute.parent() {
                std::fs::create_dir_all(parent).map_err(|error| GenError::Io {
                    path: parent.to_path_buf(),
                    detail: error.to_string(),
                })?;
            }
            std::fs::write(&absolute, body).map_err(|error| GenError::Io {
                path: absolute,
                detail: error.to_string(),
            })?;
        }
        Ok(Self { path })
    }
}

impl Drop for ScratchProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// The scratch project's compiler configuration: strict mode over every emitted declaration,
/// nothing written back.
const STRICT_TSCONFIG: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "noEmit": true,
    "target": "es2020",
    "module": "esnext",
    "moduleResolution": "bundler"
  },
  "include": ["**/*.ts"]
}
"#;

/// Writes one top-level declaration: an `interface` for object shapes, a `type` alias for
/// everything else, each preceded by its `JSDoc` when the node documents itself.
fn write_export(
    out: &mut String,
    name: &str,
    node: &Value,
    schema_id: &str,
) -> Result<(), GenError> {
    push_jsdoc(out, node, 0);
    if let (true, Some(members)) = (is_object_shape(node), node.as_object()) {
        audit(members, OBJECT_SHAPE_KEYWORDS, schema_id)?;
        if members_are_empty(node) {
            let _ = writeln!(out, "export interface {name} {{}}");
            return Ok(());
        }
        let _ = writeln!(out, "export interface {name} {{");
        push_members(out, node, 1, schema_id)?;
        let _ = writeln!(out, "}}");
    } else {
        // Non-object shapes go through the inline projector, which carries its own audit and
        // refuses nodes that are not schema objects at all.
        let expression = project_inline(node, schema_id)?;
        let _ = writeln!(out, "export type {name} = {expression};");
    }
    Ok(())
}

/// Rejects any keyword the dispatcher did not consume and the inert list does not excuse.
///
/// This is the enforcement half of "fail-closed": projection may succeed structurally while a
/// sibling keyword (`patternProperties`, `if`, `propertyNames`, …) still changes what the
/// schema accepts, so success is only legal when *every* keyword is accounted for.
fn audit(members: &Map<String, Value>, consumed: &[&str], schema_id: &str) -> Result<(), GenError> {
    for key in members.keys() {
        if !INERT_KEYWORDS.contains(&key.as_str()) && !consumed.contains(&key.as_str()) {
            return Err(GenError::UnrepresentableConstruct {
                schema_id: schema_id.to_owned(),
                construct: key.clone(),
            });
        }
    }
    Ok(())
}

/// `true` when the node describes a JSON object: an explicit `"type": "object"` or a
/// `properties` keyword without one.
fn is_object_shape(node: &Value) -> bool {
    node.get("type").and_then(Value::as_str) == Some("object") || node.get("properties").is_some()
}

/// `true` when an object node declares no properties and no open additional-properties rule,
/// so its interface body would be empty.
fn members_are_empty(node: &Value) -> bool {
    let members = node.as_object();
    let properties_empty = members
        .and_then(|members| members.get("properties"))
        .and_then(Value::as_object)
        .is_none_or(Map::is_empty);
    let open = members.is_some_and(|members| {
        matches!(members.get("additionalProperties"), Some(Value::Bool(true)))
    });
    properties_empty && !open
}

/// Writes every member of an object node at the given indent depth: one line per property
/// with its `JSDoc`, an optional-marker per `required`, and an index signature when the
/// interface is open.
fn push_members(
    out: &mut String,
    node: &Value,
    indent: usize,
    schema_id: &str,
) -> Result<(), GenError> {
    let Some(members) = node.as_object() else {
        return Ok(());
    };
    let required: Vec<&str> = members
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let padding = " ".repeat(indent);

    if let Some(properties) = members.get("properties").and_then(Value::as_object) {
        for (property_name, property_node) in properties {
            push_jsdoc(out, property_node, indent);
            let marker = if required.contains(&property_name.as_str()) {
                ""
            } else {
                "?"
            };
            let spelled = spell_identifier(property_name);
            let projected = project_inline(property_node, schema_id)?;
            let _ = writeln!(out, "{padding}{spelled}{marker}: {projected};");
        }
    }

    match members.get("additionalProperties") {
        Some(Value::Bool(true)) => {
            let _ = writeln!(out, "{padding}[key: string]: unknown;");
        }
        Some(Value::Bool(false)) | None => {}
        Some(_) => {
            return Err(unrepresentable(schema_id, "additionalProperties subschema"));
        }
    }
    Ok(())
}

/// Projects any schema node to a single-line TypeScript type expression.
///
/// Dispatch order is fixed — reference, const, enum, combinators, negation, typed shapes —
/// and whichever arm fires records the keywords it consumed, so the closing audit knows
/// exactly what is left to account for.
fn project_inline(node: &Value, schema_id: &str) -> Result<String, GenError> {
    let Some(members) = node.as_object() else {
        return Err(unrepresentable(schema_id, "non-object schema node"));
    };
    let mut consumed: Vec<&str> = Vec::new();

    let expression = if let Some(reference) = members.get("$ref") {
        consumed.push("$ref");
        project_reference(reference, schema_id)?
    } else if let Some(constant) = members.get("const") {
        // schemars pairs `const` with the constant's JSON kind (`"type": "string"`); the kind
        // refines the literal, so both keywords are consumed together.
        if members.contains_key("type") {
            consumed.push("type");
        }
        consumed.push("const");
        literal(constant, schema_id)?
    } else if let Some(variants) = members.get("enum").and_then(Value::as_array) {
        if members.contains_key("type") {
            consumed.push("type");
        }
        consumed.push("enum");
        literal_union(variants, schema_id)?
    } else if let Some(branches) = members
        .get("oneOf")
        .or_else(|| members.get("anyOf"))
        .and_then(Value::as_array)
    {
        consumed.push(if members.contains_key("oneOf") {
            "oneOf"
        } else {
            "anyOf"
        });
        // A second combinator of the pair stays unconsumed and the audit rejects it: the two
        // spell different validation semantics and one projection cannot honour both.
        union(branches, schema_id)?
    } else if let Some(conjuncts) = members.get("allOf").and_then(Value::as_array) {
        consumed.push("allOf");
        let parts = conjuncts
            .iter()
            .map(|conjunct| project_inline(conjunct, schema_id))
            .collect::<Result<Vec<_>, _>>()?;
        parts.join(" & ")
    } else if let Some(negated) = members.get("not") {
        consumed.push("not");
        complement(negated, schema_id)?
    } else {
        match members.get("type") {
            Some(Value::String(kind)) => {
                consumed.push("type");
                match kind.as_str() {
                    "string" => "string".to_owned(),
                    "integer" | "number" => "number".to_owned(),
                    "boolean" => "boolean".to_owned(),
                    "null" => "null".to_owned(),
                    "array" => {
                        consumed.push("items");
                        project_array(node, schema_id)?
                    }
                    "object" => {
                        consumed.extend_from_slice(OBJECT_SHAPE_KEYWORDS);
                        inline_object(node, schema_id)?
                    }
                    other => return Err(unrepresentable(schema_id, other)),
                }
            }
            Some(Value::Array(kinds)) => {
                consumed.push("type");
                nullable_type_array(kinds, schema_id)?
            }
            Some(other) => {
                return Err(unrepresentable_schema_node(schema_id, other));
            }
            None => return Err(unrepresentable(schema_id, "untyped schema node")),
        }
    };

    audit(members, &consumed, schema_id)?;
    Ok(expression)
}

/// `#/$defs/Name` → bare `Name`. Any other reference target crosses files or depths the
/// self-contained layout cannot express, so it is refused.
fn project_reference(reference: &Value, schema_id: &str) -> Result<String, GenError> {
    let target = reference
        .as_str()
        .and_then(|target| target.strip_prefix("#/$defs/"));
    match target {
        Some(name) if !name.is_empty() && !name.contains('/') => Ok(name.to_owned()),
        _ => Err(unrepresentable(schema_id, "$ref")),
    }
}

/// One union: branches keep document order, exact `{"type": "null"}` branches collapse into a
/// trailing `| null` (D3), and an all-null union projects to plain `null`.
fn union(branches: &[Value], schema_id: &str) -> Result<String, GenError> {
    let mut parts = Vec::new();
    let mut nullable = false;
    for branch in branches {
        let is_bare_null = branch.as_object().is_some_and(|branch| {
            branch.len() == 1 && branch.get("type").and_then(Value::as_str) == Some("null")
        });
        if is_bare_null {
            nullable = true;
            continue;
        }
        parts.push(project_inline(branch, schema_id)?);
    }
    if parts.is_empty() && !nullable {
        return Err(unrepresentable(schema_id, "union with no branches"));
    }
    if nullable {
        parts.push("null".to_owned());
    }
    Ok(parts.join(" | "))
}

/// An array-form `type` such as `["string", "null"]`: primitive kinds only, `null` last.
fn nullable_type_array(kinds: &[Value], schema_id: &str) -> Result<String, GenError> {
    let mut parts = Vec::new();
    let mut nullable = false;
    for kind in kinds {
        match kind.as_str() {
            Some("null") => nullable = true,
            Some("string") => parts.push("string".to_owned()),
            Some("integer" | "number") => parts.push("number".to_owned()),
            Some("boolean") => parts.push("boolean".to_owned()),
            _ => {
                return Err(unrepresentable(
                    schema_id,
                    "non-primitive type array member",
                ));
            }
        }
    }
    if parts.is_empty() && !nullable {
        return Err(unrepresentable(schema_id, "type array with no members"));
    }
    if nullable {
        parts.push("null".to_owned());
    }
    Ok(parts.join(" | "))
}

/// An `items`-typed array; a union or intersection element needs parentheses before the `[]`.
/// An array without `items` accepts arbitrary elements, which is precisely `unknown[]`.
fn project_array(node: &Value, schema_id: &str) -> Result<String, GenError> {
    match node.get("items") {
        None | Some(Value::Null) => Ok("unknown[]".to_owned()),
        Some(items) => {
            let element = project_inline(items, schema_id)?;
            if element.contains(" | ") || element.contains(" & ") {
                Ok(format!("({element})[]"))
            } else {
                Ok(format!("{element}[]"))
            }
        }
    }
}

/// An anonymous object type in property or branch position, compact single-line form:
/// `{ part_kind: "text"; text?: AiText }`. Member documentation attaches to named
/// declarations, where `JSDoc` can live; discriminator branches repeat no prose.
fn inline_object(node: &Value, schema_id: &str) -> Result<String, GenError> {
    let mut inner = String::new();
    push_members(&mut inner, node, 0, schema_id)?;
    if inner.is_empty() {
        // A closed object accepting no known members: inhabitable only by the empty record.
        return Ok("Record<string, never>".to_owned());
    }
    let compact = inner.lines().map(str::trim).collect::<Vec<_>>().join(" ");
    Ok(format!("{{ {compact} }}"))
}

/// The two `not` patterns the committed contracts use, mapped soundly; any other negation is
/// outside the subset. `{not: {enum: [strings]}}` accepts any token outside the listed kinds,
/// which is a subset of `string`; `{not: {type: object}}` is exactly [`NON_OBJECT_JSON`].
fn complement(negated: &Value, schema_id: &str) -> Result<String, GenError> {
    let Some(inner) = negated.as_object() else {
        return Err(unrepresentable(schema_id, "not over a non-object schema"));
    };
    if inner.len() == 1 && inner.get("type").and_then(Value::as_str) == Some("object") {
        return Ok(NON_OBJECT_JSON.to_owned());
    }
    let enumerated = inner.get("enum").and_then(Value::as_array);
    let all_strings = enumerated
        .is_some_and(|variants| !variants.is_empty() && variants.iter().all(Value::is_string));
    if all_strings {
        return Ok("string".to_owned());
    }
    Err(unrepresentable(schema_id, "not"))
}

/// A union of literal types, one per `enum` entry.
fn literal_union(variants: &[Value], schema_id: &str) -> Result<String, GenError> {
    if variants.is_empty() {
        return Err(unrepresentable(schema_id, "empty enum"));
    }
    let branches = variants
        .iter()
        .map(|variant| literal(variant, schema_id))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(branches.join(" | "))
}

/// A literal type: quoted string, integer, boolean or `null`. Composite constants have no
/// single-token TypeScript literal worth emitting, so they refuse rather than approximate.
fn literal(value: &Value, schema_id: &str) -> Result<String, GenError> {
    match value {
        Value::String(text) => serde_json::to_string(text)
            .map_err(|_| unrepresentable(schema_id, "unrepresentable const string")),
        Value::Number(number) if number.is_i64() || number.is_u64() => Ok(number.to_string()),
        Value::Bool(flag) => Ok(flag.to_string()),
        Value::Null => Ok("null".to_owned()),
        _ => Err(unrepresentable(schema_id, "composite const")),
    }
}

/// Appends the `JSDoc` for a node: its description verbatim, then a `Format:` note when the
/// node carries a format keyword (D3). A node with neither produces nothing.
fn push_jsdoc(out: &mut String, node: &Value, indent: usize) {
    let description = node
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .replace("*/", "* /");
    let format_note = node
        .get("format")
        .and_then(Value::as_str)
        .map(|format| format!("Format: {format}."));
    if description.is_empty() && format_note.is_none() {
        return;
    }
    let padding = " ".repeat(indent);
    let _ = writeln!(out, "{padding}/**");
    for line in description.lines() {
        let _ = writeln!(out, "{padding} * {line}");
    }
    if let Some(note) = format_note {
        if !description.is_empty() {
            let _ = writeln!(out, "{padding} *");
        }
        let _ = writeln!(out, "{padding} * {note}");
    }
    let _ = writeln!(out, "{padding} */");
}

/// Quotes a property name that is not a legal bare TypeScript identifier.
fn spell_identifier(name: &str) -> String {
    let well_formed = name.chars().enumerate().all(|(index, character)| {
        character.is_ascii_alphabetic()
            || character == '_'
            || character == '$'
            || (index > 0 && character.is_ascii_digit())
    });
    if well_formed {
        name.to_owned()
    } else {
        serde_json::to_string(name).unwrap_or_else(|_| format!("{name:?}"))
    }
}

/// Builds the fail-closed error naming the construct that refused projection.
fn unrepresentable(schema_id: &str, construct: &str) -> GenError {
    GenError::UnrepresentableConstruct {
        schema_id: schema_id.to_owned(),
        construct: construct.to_owned(),
    }
}

/// Same, for a `type` keyword holding something that is neither string nor array.
fn unrepresentable_schema_node(schema_id: &str, offender: &Value) -> GenError {
    GenError::UnrepresentableConstruct {
        schema_id: schema_id.to_owned(),
        construct: format!("type ({offender})"),
    }
}
