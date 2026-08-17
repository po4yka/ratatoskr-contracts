# Security Policy for Ratatoskr Contracts

> Status: Proposed  
> Last reviewed: 2026-08-17

There is no supported production release yet. Report vulnerabilities privately through GitHub private vulnerability reporting when enabled or another established private channel. Do not publish secrets, real tokens, personal exports, private message payloads, or exploit details in public issues.

Security review is required for identity fields, credential-bearing schemas, authorization context, blob references, signing/encryption metadata, generated code, parser changes, and compatibility rules.

Baseline:

- Fixtures are synthetic and scanned for secrets/PII.
- Canonical schemas define bounds, nullability, discriminators, and privacy classification.
- Generators are pinned, reproducible, and reviewed as supply-chain code.
- Breaking changes use coordinated rollout; do not create silent consumer downgrade.
- Schemas never expose storage paths, encrypted secret bytes, or internal error details without an explicit design.
