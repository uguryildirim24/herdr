+++
verdict = "MERGE"
round = "r21"
candidate = "565c5db941e94f4192d023946589b01d8628bfb4"
manifest_hash = "dd8f4f0b5afa623a9db95db04a44ea28f4497199a73c75a68837fc75aabb7504"
policy_hash = "7cf667dea9748fe2844267e971e613ff74b94a3ef0af9df5ee974d759cb7e9da"
gates = []
+++

# Round r21 review

## Verdict

MERGE. The machine-qualified fold targets, context menu, compact status-mark fold, dot stack,
and same-machine drag geometry are coherent and covered for colliding pane ids on Local and
`oci`.

## Lane t-0042

The lane correctly reuses the normal agent-row renderer and collapse keys in the aggregate
sidebar. Its tests cover both machines, padded live-shaped rows, scrolling, context-menu folding,
compact folding, stacks, target marks, and cross-machine rejection.

I fixed one integration defect in `review(client): keep aggregate drag on the active machine`.
The lane tried to send the drop method directly to an inactive machine. The client runtime cancels
commands for any machine that does not own the active shell surface, and that server rejects such
commands as `surface_inactive`, so the unit-level action assertion could not work in the running
client. Aggregate drag now follows the existing workspace behavior: rows on the active machine
can drag; a gesture on another machine activates and focuses it, after which a drag uses its active
command surface. Tests now establish that distinction and still prove remote-machine nesting after
activation.

The compact aggregate list intentionally remains one glyph pair per row and therefore does not
show the expanded panel's hidden-status stack. This does not block the requested fold and drag
behavior.

## Round gates

The brief listed no gates, so the verdict gate list is empty.

## Additional validation

```text
$ cargo fmt --check
(no output; exit 0)
```

```text
$ cargo clippy --all-targets --locked -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.94s
```

```text
$ cargo test --locked -p herdr client::shell
running 388 tests
...
test result: ok. 386 passed; 0 failed; 2 ignored; 0 measured; 3141 filtered out
```

```text
$ cargo build --release --locked
    Finished `release` profile [optimized] target(s) in 43.43s
```

```text
$ just bench-render-scale
...
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 3523 filtered out
```

The fixed-geometry profile measured client-shell composition from 1 to 15 panes at 123 to 132
microseconds for background workspaces and 120 to 131 microseconds for active panes. The navigator
profile measured 130.9 to 136.8 microseconds per frame.
