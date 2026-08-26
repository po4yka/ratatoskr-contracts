## 1. Reproduce the security finding

- [x] 1.1 Run the local zizmor audit before the change and confirm `contracts.yml` reports the two template-expansion findings at the package archive command.
- [x] 1.2 Run `cargo test -p ratatoskr-contractsc --test determinism` before the change and confirm the cardinality pins fail because the registered set is 28/56 rather than 27/54.
- [x] 1.3 Run `cargo test -p ratatoskr-contractsc --test metadata` before the change and confirm M-2 finds the out-of-order social root and M-8 finds 11 events rather than 10.

## 2. Repair the shell boundary

- [x] 2.1 Construct the package archive filename from `GITHUB_SHA` in the shell script while preserving the SHA-named uploaded artifact; verify the workflow diff is limited to that boundary.
- [x] 2.2 Update all stale determinism cardinality pins to 28 TypeScript roots and 56 generated artifacts; verified all 18 determinism tests pass and no generator output changes.
- [x] 2.3 Restore lexical root-registry order and update the registered-event pin to 11; verified all 11 metadata tests pass and generated artifacts remain current.

## 3. Verify the gate

- [ ] 3.1 Re-run the local zizmor audit and verify it reports no findings; then run the documented repository gate and observe the hosted zizmor check after integration.
