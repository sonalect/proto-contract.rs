# Dependency list order

When adding, removing, or bumping a dependency, put it in its sorted place.
Do not append at the bottom.

## Cargo.toml

- `[workspace.members]`: alphabetical by path.
- `[workspace.dependencies]`, `[dependencies]`, `[dev-dependencies]`,
  `[build-dependencies]`, per-target tables: alphabetical by package name,
  the path crates (`contratto`, `protoc-gen-contratto-rust`) sorted in
  with the rest. Optional crates stay in that same sort.

```toml
# BAD — connectrpc dumped at the bottom
buffa = { workspace = true }
tokio = { workspace = true, optional = true }
connectrpc = { workspace = true, optional = true }

# GOOD
buffa = { workspace = true }
connectrpc = { workspace = true, optional = true }
tokio = { workspace = true, optional = true }
```

## BUILD.bazel

`deps` / `proc_macro_deps` lists: alphabetical. Workspace `//rust/…` first,
then `@contratto_cargo//:…`. `bazel run //bazel:format` sorts what
buildifier can.

Does not replace `dependency-quarantine.md` or the crate_universe repin
(`cargo-bazel-lock.md`).
