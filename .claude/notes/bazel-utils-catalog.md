# bazel_utils catalog entry for the plugin

Deferred by the owner on 6 October 2026: the repository stays private for
now, and bazel_utils downloads plugin binaries without authentication, so
GitHub answers 404 for a private repository's release assets.

## State

- Prepared, not committed, on the local branch `protoc-gen-contract-rust`
  of `../bazel_utils` (branched from `main` at v0.2.12):
  `protoc/plugins/protoc-gen-contract-rust/` (`registry.bzl`, kind `file`,
  six platforms; `BUILD.bazel`), the `buf` re-export, `defs.bzl`,
  `protoc/plugins/BUILD.bazel`, the `protoc.plugin` fallback and
  `use_repo` in `protoc/MODULE.bazel`, the root `protoc_plugins`
  filegroup, README (both plugin tables, the name list, the example), and
  CHANGELOG (Unreleased: Added; the missing `[0.2.12]` link definition).
- The six sha256 values are filled in from the v0.1.0 Release's
  `checksums-sha256.txt` (published 6 October 2026); a downloaded linux
  binary matched its sum.

## Repository renamed

The owner renamed the repository from `sonalect/protoc-gen-contract-rust`
to `sonalect/proto-contract.rs` on 7 October 2026; this repository's files
and `origin` follow. Deferred by the owner ("later"): the old URL is still
in `../bazel_utils` (`README.md`, and the release download URLs in
`protoc/plugins/protoc-gen-contract-rust/registry.bzl` on the local
branch) and in `../knowqore` (`Cargo.toml`, `Cargo.lock`,
`.claude/rules/dependency-quarantine.md`, `.claude/notes/format-pilot.md`).
GitHub redirects the old URL meanwhile. The plugin binary keeps its name;
the runtime crate is `protocontract` since 7 October 2026 (it was
`contract`), so Knowqore's dependency and `contract::` paths change with
its next pin.

## How to finish

1. Make `sonalect/proto-contract.rs` public (owner), and point the
   bazel_utils download URLs at it (see above).
2. In `../bazel_utils`: `bazel test //... --keep_going` (it fetches all six
   binaries and checks the sums), commit with `-s`, bump to 0.2.13 in every
   `MODULE.bazel` and README, annotated tag, `gh release create`.
3. Here: nothing to change; Knowqore then pins
   `protoc.plugin(name = "protoc-gen-contract-rust", version = "v0.1.0")`.

**Why:** a catalog entry that cannot be fetched breaks bazel_utils' own CI
(its root build fetches every plugin) and every consumer that pins it.
