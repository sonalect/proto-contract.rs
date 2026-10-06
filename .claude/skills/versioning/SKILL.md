---
name: versioning
description: >-
  Bump the Contract crates and Bazel module versions in lockstep
  (Rust-style 0.x or SemVer after 1.0.0), update CHANGELOG.md, create the
  matching GitHub tag, whose CI workflow tests six platforms and publishes
  the GitHub Release with the plugin binaries. Use when releasing, tagging, publishing a GitHub release,
  bumping version, editing MODULE.bazel or workspace package version, or
  adding a CHANGELOG section.
---

# Versioning

One release version for the whole repository: the runtime crate
`contract`, the plugin crate `protoc-gen-contract-rust`, and the Bazel
module. The status line of `DESIGN.md` is **not** this number.

## Scheme

Versions are `MAJOR.MINOR.PATCH`. `module(version = "0.0.1")` and Cargo
`version = "0.0.1"` have **no** `v`. The git tag **does**: `v0.0.1`.

If **major is 0** (Rust approach, also stated in `CHANGELOG.md`):

- Breaking change → bump **minor** (`0.0.1` → `0.1.0`)
- Bugfix or compatible feature → bump **patch** (`0.0.1` → `0.0.2`)

If **major > 0** (standard SemVer):

- Breaking → **major**
- Compatible features → **minor**
- Bugfixes → **patch**

Do not skip numbers. Do not jump to `1.0.0` unless the owner explicitly
wants a stable public API.

**Breaking** means consumer-visible (`.claude/rules/consumers.md`): a
renamed or removed generated item, a changed generated signature or module
layout, a removed or re-meant plugin parameter or default, a changed
runtime type, or a changed `Status` mapping. Toolchain and CI pins (Bazel,
rustc) are a patch (0.x) or minor (≥1.0) unless they force a breaking
change. A bump of the pinned buffa or connect-rust that changes the
generated code is breaking.

Pinned `bazel_utils_*` / `rules_rust` versions are **not** this number.

## Lockstep files

Set the same `MAJOR.MINOR.PATCH` everywhere:

- Root `MODULE.bazel`: `module(version = …)`
- Root `Cargo.toml`: `[workspace.package] version` (members use
  `version.workspace = true`)
- `CHANGELOG.md`: move `## [Unreleased]` notes into `## [MAJOR.MINOR.PATCH] -
  YYYY-MM-DD`; add the version to `## Links` and the reference definitions
  (`[Unreleased]` compare from the new tag, `[MAJOR.MINOR.PATCH]` release URL)

Refresh `MODULE.bazel.lock` if it changes.

## Release

Only when the owner asked to release, and only from `main`:

1. Commit the version bump and `CHANGELOG.md` on `main` and push it.
2. Run the workflow by hand on `main` and wait for it: the same six
   platforms and binaries, no Release, and it saves the Bazel caches the
   tag run will restore.

   ```bash
   gh workflow run Release --ref main
   gh run watch "$(gh run list --workflow Release --branch main --limit 1 --json databaseId -q '.[0].databaseId')"
   ```

3. Create the annotated tag with the release notes as its message and push
   it, exactly as `.claude/rules/release-tag.md` says (a file,
   `--cleanup=verbatim`, read back before the push). Do not backfill older
   tags.

The tag starts `.github/workflows/release.yml`; nothing else does. It:

1. fails at once unless the tag is annotated, points at a commit on
   `main`, has a message body, and is `v` plus the version in `Cargo.toml`
   and `MODULE.bazel`, with `## [MAJOR.MINOR.PATCH]` in `CHANGELOG.md`;
2. runs `bazel build //...` and `bazel test //...` on six platforms
   (`linux_amd64`, `linux_arm64`, `darwin_amd64`, `darwin_arm64`,
   `windows_amd64`, `windows_arm64`), and on each builds the plugin's
   static release binary with Cargo (`--profile dist`);
3. when all six pass, publishes the GitHub Release for the tag: the six
   binaries, named `protoc-gen-contract-rust-{tag}-{file}` (`DESIGN.md`
   §9), and a `SHA256SUMS` file; the tag message's subject is the title,
   its body the notes. Not a draft, not a prerelease.

Do not run `gh release create` by hand. A tag whose workflow failed has no
Release: fix the cause on `main`, delete the tag locally and on `origin`,
and tag the fixed commit.

Watch the run with `gh run watch` and return the Release URL when it
succeeds. The `registry.bzl` entry in `bazel_utils` (URL template and the
sha256 of each platform, from `SHA256SUMS`) is a change in that
repository: prepare it only when the owner asks.

Examples: `v0.0.1`, `v0.0.2`, `v0.1.0`.
