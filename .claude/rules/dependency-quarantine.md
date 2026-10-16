# Dependency quarantine

Before adding or bumping an **external** dependency in any language
(crates.io, PyPI, npm, Go modules, git tags, or any other registry), look
up the **publish time of the exact version**. Do not use a version
published less than **2 days** ago.

This is a quiet quarantine against freshly published malicious releases.

- Check the registry API or package page for that version. Do not trust
  memory, download mirrors, or lockfile dates. The `scout` agent can do
  the lookup.
- If the wanted version is too new, take the newest version that is at
  least 2 days old, or wait. Do not add the package if no qualifying
  version exists.
- Applies to new packages and to version bumps of existing ones, git-tag
  pins of forks included. Does not apply to this repo's own workspace /
  path dependencies.

## Exemption: the `sonalect` organization

The quarantine does not apply to a project that lives in the same GitHub
organization as this repository (`github.com/sonalect/*`, `bazel_utils` for
one). Take its newest release at once; do not wait two days. The owner
decided this on 12 October 2026, because the organization's own releases are
not a third-party supply chain.

- Still read the release notes and the diff since the pinned tag for
  breaking changes, and run the gate (`commit-after-tests.md`).
- The exemption covers the organization's own repositories only. Whatever
  such a project pulls in from outside (crates, npm, other registries) stays
  under the quarantine when this repository adds or bumps it directly.

## Cargo: ranges in `Cargo.toml`, exact versions in `Cargo.lock`

Decided by the owner on 16 October 2026, for publishing on crates.io:
`[workspace.dependencies]` takes caret ranges (`buffa = "0.9.2"`), never
`=` pins, because an exact pin in a published library conflicts with a
consumer on any other version. The quarantine then lives in `Cargo.lock`:

- A plain `cargo update` takes the newest version in every range, however
  fresh. Never run it bare. Move one crate at a time, to a version checked
  against the quarantine: `cargo update -p <crate> --precise <version>`.
- Read `git diff Cargo.lock` after any Cargo command that may resolve
  (`cargo update`, a new dependency, a changed feature): every version that
  moved must clear the 2 days.
- The range's lower bound is the version we test with. Raise it when the
  code needs a newer API, not on every lock bump.
