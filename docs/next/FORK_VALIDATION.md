# Fork verification record

This record covers Rolf's working copy of the `agent-parent-nesting` fork during publication cleanup. It is not an upstream release certification or a completed human review. Checks were run by coding agents on macOS ARM64.

## Scope review

The scope review kept documentation, package notice metadata, ignore rules, and tree-level privacy fixes. Runtime source, dependencies, tests, build code, and the upstream `LICENSE` match HEAD. No runtime fix was needed. Broad ignore rules for data, datasets, outputs, and artifacts were removed so they do not hide source inputs or evidence.

The cached `cargo build --release --locked --offline` passed with Rust/Cargo 1.96.1 and Zig 0.16.0. The executable reported `herdr 0.9.1`. No empty-cache build or network clone was attempted.

The README shell blocks ran with an isolated home and `/bin/sh`. Short relative config paths inside ignored `target/` avoided the review directory's socket path limit. First-run setup, pane listing, parent assignment, pane inspection, clearing, detach, reattach, and server stop passed. Two shell panes used synthetic agent reports. No paid agent or native agent resume was exercised. One attempt to reuse a PTY exited before reattachment. Cold attach and live reattach passed with separate controlling PTYs. All servers started by this review were stopped.

The version and preview documentation validators, configuration reference check, detection manifest check, and draft translation parity check passed. `cargo fmt --check` and `git diff --check` passed. `cargo package --list --allow-dirty --offline` included `NOTICE`; this was a listing, not a package build. A redacted Gitleaks directory scan found six matches. Inspection confirmed four environment-variable references in vendor workflows and two code fixtures or assignments, not secret values. A supplemental text scan found no listed sensitive Rolf information. No tracked file exceeded 5 MB.

`just test` and `just check` were not rerun. Their existing Git fixtures write `.git`, which this scope review prohibits. Windows SDK setup, upstream installers, release workflows, and paid agent setup were skipped. No new tests were added. Earlier test counts, handoff observations, and limitations below are from the earlier cleanup, not new results from the scope review.

## Build

Tools: Rust 1.96.1, Cargo 1.96.1, Zig 0.16.0, cargo-nextest 0.9.143, and Bun 1.4.2. Zig and Bun were downloaded into ignored `target/` directories. Download integrity was checked. Cargo dependencies and Zig packages were already cached.

The release build passed before and after the cleanup:

```bash
cargo build --release --locked --offline
./target/release/herdr --version
```

The executable reported `herdr 0.9.1`. The build used `ZIG` to select Zig 0.16.0. `CARGO_HOME`, `CARGO_TARGET_DIR`, Zig caches, and temporary files were kept inside ignored `target/`. An empty-cache clean clone was not exercised. The README gives the normal source-build commands, which need network access on the first build.

`cargo fmt --check` passed. `cargo package --list --allow-dirty` included the new root `NOTICE`. That listing is not a successful package build. The fork build requires the full clone, including vendored sources.

## Existing tests

`just test` did not pass. With an absolute temporary directory inside the review worktree, nextest stopped after six socket tests failed with `local socket name length exceeds capacity of sun_path of sockaddr_un`. The review directory is longer than macOS Unix socket paths permit.

A second `just test` attempt with a short relative `TMPDIR` passed the socket stage, then stopped at a cwd assertion. A full diagnostic nextest run used the same relative directory:

```bash
cargo nextest run --locked --no-fail-fast \
  --status-level fail --final-status-level fail \
  --failure-output final --success-output never
```

That run reported 3,622 passed, 49 failed, and 7 skipped. Two passing tests were marked leaky by nextest. Failures included cwd expectations, symlink paths, disposable Git fixture paths, plugin startup, and manifest-cache checks. A relative `TMPDIR` is not a supported fix: several existing tests assume an absolute path. These results do not prove that every failure is environmental. Repeat the full suite with a normal short absolute temporary directory and investigate remaining failures before release. Existing full-suite tests also create disposable Git fixture repositories, which matters in a review that prohibits Git writes.

A focused run of existing lineage, sidebar, resume, and restore tests passed with the absolute worktree-local temporary directory:

```bash
cargo nextest run --locked \
  -E 'test(agent_parents::) | test(client::shell::agent_tree::) | test(client::shell::tests::agent_sidebar::) | test(agent_resume::) | test(persist::restore::)' \
  --status-level fail --final-status-level fail \
  --failure-output final --success-output never
```

All 97 selected tests passed. This is a subset, not a replacement for the full recipe.

These existing commands also passed:

```bash
just integration-assets-test
just ui-hot-path-architecture-test
node scripts/docs/versions.mjs check
node scripts/docs/preview.mjs check
python3 scripts/config_reference_check.py
python3 scripts/agent_detection_manifest_check.py
python3 scripts/docs_translation_parity.py --docs-root docs/next/website/src/content/docs
```

The integration asset recipe ran 39 tests. The architecture recipe ran 6 tests. The documentation snapshot checks validated 22 version snapshots and the preview snapshot. The maintenance and documentation contract recipes after nextest were not reached by `just test`. `just check`, Windows cross-compilation, release performance checks, and package installation were not completed.

## Isolated runtime checks

The release executable ran in a pseudo-terminal with an isolated home, config, state directory, and `/bin/sh`. Upstream version and manifest checks were disabled in that config. Socket paths were short relative paths inside the worktree; no installed session was used.

Observed:

- The TUI attached. A CLI pane split created a second shell pane.
- `pane report-agent` supplied synthetic custom agent reports. No paid or real agent process was launched.
- `agent set-parent w1:p2 w1:p1` succeeded while those reports were active. `pane get` returned the parent token. The README's clear command also succeeded before handoff.
- `ctrl+b q` detached the client while the server stayed available.
- `server restart` succeeded. The child shell PID, pane ID, and parent token stayed the same. The terminal ID changed, so terminal ID stability is not claimed.
- Stopping and starting the isolated session restored two shell panes and the saved parent token. This was a new process restore, not native conversation resume.

The synthetic agent reports were absent after live handoff. A later `agent set-parent w1:p2 --clear` returned `agent_not_found`, although the pane still had its parent token. This remains a limitation of the observed sequence, not a verified result for official integrations. All isolated review servers were stopped afterward.

Cross-machine trees, SSH reconnection, actual agent sidebar interaction, official integration lifecycle preservation, native conversation resume, Linux runtime behavior, Windows, and graphical agent detection were not exercised. No fork performance or scientific results are claimed.

## Inherited-environment isolation follow-up

The README launch and second-terminal shell blocks were exercised with inherited socket, client socket, config, named-session, nested-session, pane, and remote-client variables. Both blocks clear that context before setting the fork's config and state directories. A separate disposable named server represented an existing session; no installed session was contacted.

The shell blocks were run unchanged, with a synthetic `HOME` and short relative paths inside ignored `target/` to stay below macOS socket path limits. Observed:

- The fork launch created a separate server. The original server still answered, and its config was unchanged.
- First-run setup completed in a pseudo-terminal. The second-terminal commands reached the fork panes.
- Repeating config creation failed without overwriting the existing fork config.
- The README's list, set-parent, get, and clear commands succeeded with synthetic agent reports.
- Detach and reattach succeeded after cleanup, despite the inherited nested-session marker.
- `server stop` from the cleaned second terminal stopped only the fork. The original disposable server still answered afterward.

Both disposable servers were stopped. The cached offline release build and the documentation snapshot, configuration, detection-manifest, and draft translation-parity validators passed again. A repeated redacted directory scan returned the same six previously inspected false positives. Supplemental tracked-file privacy checks found no confirmed listed sensitive Rolf information. The full test recipe was not rerun for this documentation correction. The earlier test and runtime limits still apply. Command output remains ignored under `target/cleanup/` and `target/isolation-*/`.

## Privacy and publication

A redacted Gitleaks directory scan excluded ignored build products and found six matches. Inspection found only false positives: four workflow references to an environment-provided notarization key, a keybinding fixture, and a terminal mode assignment. A supplemental tracked-file scan found no confirmed credentials or listed sensitive Rolf information. The upstream screenshot and social-card image were inspected; neither showed listed sensitive Rolf information.

The upstream Apache-2.0 `LICENSE` is unchanged. No required vendor sources, maintained tests, release snapshots, or component licenses were removed. No shelved runtime feature was identified for deletion. The installation-day migration narrative was removed from all three draft session-state documents, with related migration wording removed from configuration drafts.

This scan is not a guarantee that the repository is free of sensitive information. Historical `tasks/` files are outside the cleaned tree and still need a separate publication review. Earlier upstream history was not audited. Rolf must review history, the final diff, fork release settings, and the test gaps before making the repository public.

Detailed local command output remains ignored under `target/cleanup/`. No raw terminal logs, credentials, datasets, or local workstation paths were added to tracked documentation.
