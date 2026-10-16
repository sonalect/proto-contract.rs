# Publishing to crates.io

Deferred by the owner on 16 October 2026 ("next time"). Nothing is
published; consumers take `protocontract` as a git dependency on a tag.

## State (checked 16 October 2026, at 0.3.0)

- `protocontract` and `protoc-gen-protocontract-buffa` are free names on
  crates.io.
- `cargo package -p <crate>` packages and verifies both; `cargo doc`
  builds `protocontract` with no warning.
- `rust/golden` and `examples/glossary/*` already have `publish = false`.

## Before the first publish

1. `protocontract` pins its dependencies exactly (`buffa = "=0.9.2"`,
   `tokio = "=1.53.3"`, `futures-core = "=0.3.34"`). A published library
   with exact pins conflicts with a consumer on any other version. The
   owner decides whether the library takes ranges (`Cargo.lock` keeps our
   builds exact and under the quarantine); that changes the pinning rule.
2. README: crates.io breaks relative links (`examples/glossary`,
   `DESIGN.md`); make them absolute GitHub URLs. Optionally a short README
   of its own for `protocontract`.
3. `keywords` (at most 5) and `categories` for both crates.
4. `protocontract`: `[package.metadata.docs.rs] all-features = true`, so
   docs.rs shows the `tokio`-gated `Offload` and `Blocking`.
5. `exclude = ["BUILD.bazel"]` in both packages.
6. A `.bazelignore` with `target`: `cargo package` unpacks each crate into
   `target/package/<crate>-<version>/` with its `BUILD.bazel`, and
   `bazel test //...` then runs those copies as extra packages (seen on
   16 October 2026: 21 tests instead of 17).

## How

- First publish by hand: crates.io login through GitHub, an API token,
  `cargo login`, `cargo publish --dry-run`, then `cargo publish -p
  protocontract` and `-p protoc-gen-protocontract-buffa`. A version can
  only be yanked, never replaced; the name stays with the account.
- Afterwards: crates.io Trusted Publishing from the release workflow (no
  stored token). It is configured on a crate that already exists.
- docs.rs builds the documentation of every published version by itself;
  no GitHub Pages site is needed.

**Why:** the list was worked out once with the owner; start from it.
