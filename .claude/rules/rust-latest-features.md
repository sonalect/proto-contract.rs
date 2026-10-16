---
paths:
  - "rust/**/*.rs"
---

# Latest Rust features

Two floors, decided by the owner (16 October 2026), both edition 2024:

- **Code a consumer compiles**: `rust/protocontract`, the code the plugin
  **emits**, `rust/golden`, and `examples/`. Floor: `[workspace.package]
  rust-version` in the root `Cargo.toml` (1.92, the oldest rustc that
  compiles the generated bidi-streaming handles). The release workflow
  checks it with that rustc, and clippy's `incompatible_msrv` flags a
  newer std API.
- **The plugin's own source** (`rust/protoc-gen-protocontract-buffa`): its
  `rust-version`, the pinned toolchain (`rust-toolchain.toml`). A binary we
  build; consumers never compile it.

The plugin's source may use the newest features; the Rust it writes into
`quote!` output may not go above the first floor. Write for the floor that
applies. Do not write as if MSRV were an older rustc from training data.

When writing or changing Rust, **look up** language, std, and Cargo features
stabilized after your knowledge cutoff and up through the floor that
applies. Training data is stale. Do not rely on memory for what is stable.

Sources: [Rust release notes](https://doc.rust-lang.org/stable/releases.html),
[Rust blog](https://blog.rust-lang.org/), and current std docs. Check the last
several stable releases (1.96, 1.97, 1.98, … — whatever is current), not only
the newest one.

Use a newer **stable** feature when it makes this code shorter, clearer, or
more efficient (including replacing a crate or a hand-rolled helper with std).
Skip it when it does not help, changes semantics, or would require an
unrelated mass-refactor. Nightly/`feature` gates are forbidden unless the
crate already uses them.
