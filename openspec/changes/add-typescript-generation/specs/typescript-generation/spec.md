# TypeScript Generation

## Purpose

Every published JSON Schema artifact gains a deterministic TypeScript declaration counterpart under `generated/typescript/`, produced by `cargo contracts generate` and verified by `cargo contracts check`. This gives TypeScript consumers a typed view of every contract without hand-written drift-prone mirrors, while keeping a single naming authority (`contracts.toml`) and the same provenance rigor as the JSON Schema family.

## ADDED Requirements

### Requirement: Every contract root type has a TypeScript projection

After running `cargo contracts generate`, every contract root type declared in `contracts.toml` SHALL have exactly one corresponding `.ts` artifact whose path mirrors its JSON Schema path one-to-one: the leading `schemas/` segment replaced by `generated/typescript/` and the trailing `.schema.json` suffix replaced by `.ts`. Each artifact SHALL export the root type named after the final segment of its schema identifier, followed by every embedded definition exported as a named type in sorted order and referenced bare. Artifacts SHALL be self-contained: no imports between files, no use of the `any` type.

#### Scenario: Artifact set parity after generation

- **WHEN** `cargo contracts generate` completes on the repository metadata that defines N root types with JSON Schema outputs
- **THEN** exactly N `.ts` files exist under `generated/typescript/`, each path mirroring its `schemas/` counterpart by the documented derivation

#### Scenario: Exported type name follows the schema identifier

- **WHEN** a generated artifact's schema identifier ends with a given final segment
- **THEN** the artifact exports a type with exactly that name as its root export

### Requirement: TypeScript generation is byte-deterministic

Repeated runs of `cargo contracts generate` on identical inputs SHALL produce byte-for-byte identical TypeScript artifacts. Generated files SHALL contain no timestamps or other run-dependent values, and all ordering inside an artifact (members, definition exports, union branches) SHALL be stable across inputs that differ only in irrelevant ways.

#### Scenario: Repeat generation is byte-identical

- **WHEN** `cargo contracts generate` runs twice against unchanged repository state
- **THEN** every file under `generated/typescript/` is byte-for-byte identical between the two runs

#### Scenario: Output contains no timestamps

- **WHEN** any generated TypeScript artifact is inspected
- **THEN** it contains no timestamp-formatted values or other content that changes between runs performed at different times

### Requirement: Each TypeScript artifact carries a provenance header

Each generated `.ts` artifact SHALL begin with a provenance header rendered as a leading block comment containing the marker line first, then the same nine members as the JSON Schema family (canonical source, contract identifier, contract major version, generator name and version, schemars version, source digest, validation note), with no timestamp member. The source digest SHALL be the SHA-256 over the header-less body of the artifact, recomputable by stripping everything up to and including the closing comment delimiter.

#### Scenario: Header carries the provenance members

- **WHEN** a generated TypeScript artifact is inspected
- **THEN** its leading block comment contains the generated-file marker line followed by all nine provenance members and no timestamp member

#### Scenario: Tampered body is distinguishable

- **WHEN** any byte of an artifact's header-less body is modified after generation
- **THEN** recomputing the SHA-256 over the stripped body fails to match the digest recorded in the header

### Requirement: Drift check treats TypeScript artifacts as managed outputs

`cargo contracts check` SHALL classify every expected TypeScript artifact exactly as it classifies JSON Schema artifacts: a deleted or never-generated artifact is missing, an artifact differing from current output is stale, a hand-edited artifact failing its recorded digest is tampered, and an unexpected `.ts` file under `generated/typescript/` is orphaned. All four findings SHALL fail the command non-zero.

#### Scenario: Deleted declaration is reported missing

- **WHEN** `cargo contracts check` runs after a generated `.ts` artifact is removed from the tree
- **THEN** the run reports that artifact as missing and exits non-zero

#### Scenario: Stale regeneration is reported

- **WHEN** `cargo contracts check` runs while a committed `.ts` artifact differs from what current generation would produce
- **THEN** the run reports that artifact as stale and exits non-zero

#### Scenario: Hand-edited declaration is reported tampered

- **WHEN** `cargo contracts check` runs after manual edits to a committed `.ts` artifact's body
- **THEN** the run reports that artifact as tampered and exits non-zero

#### Scenario: Orphaned declaration is reported

- **WHEN** `cargo contracts check` runs with an unexpected `.ts` file present under `generated/typescript/`
- **THEN** the run reports that file as orphaned and exits non-zero

### Requirement: Emitted declarations are strict-mode compilable

Generated artifacts SHALL compile under TypeScript strict mode with no emitted errors and without the `any` type. When a construct in a normalized schema cannot be projected to sound TypeScript, `cargo contracts generate` SHALL abort naming the offending contract's schema identifier rather than emit an unsound approximation, and SHALL not leave a declaration file behind for that contract.

#### Scenario: Generated tree compiles in a scratch project

- **WHEN** all generated TypeScript artifacts are compiled together under strict-mode TypeScript configuration
- **THEN** compilation succeeds with zero errors

#### Scenario: Unrepresentable construct aborts generation

- **WHEN** a contract's normalized schema contains a construct outside the supported mapping subset
- **THEN** `cargo contracts generate` exits with an error naming that contract's schema identifier and no declaration file for that contract exists afterward

### Requirement: A compile verification verb validates emitted declarations

The generator CLI SHALL provide a `check-typescript` verb that compiles the current generated TypeScript artifacts under a strict-mode throwaway project. It SHALL exit zero when compilation succeeds, non-zero with the compiler diagnostic when a type error occurs, and non-zero with actionable install-or-override guidance when no TypeScript compiler can be located. Compiler discovery SHALL honor an explicit environment override before falling back to a local resolution strategy, and the verb SHALL NOT be part of the repository gate command list.

#### Scenario: Declarations compile cleanly

- **WHEN** `cargo contracts check-typescript` runs with a working TypeScript compiler available
- **THEN** it exits zero after compiling the current generated artifacts

#### Scenario: Type error is surfaced

- **WHEN** `cargo contracts check-typescript` runs against artifacts that fail strict-mode compilation
- **THEN** it exits non-zero and prints the compiler's diagnostic

#### Scenario: Missing compiler is actionable

- **WHEN** `cargo contracts check-typescript` runs where neither the environment override nor local resolution yields a compiler
- **THEN** it exits non-zero with a message describing how to install a compiler or point the override at one
