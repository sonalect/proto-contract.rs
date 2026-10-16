# Publishing to crates.io

The owner decided on 16 October 2026 to publish `protocontract` and
`protoc-gen-protocontract-buffa` on crates.io with the release after
0.3.0. Nothing is published yet.

## Done (16 October 2026, Unreleased)

- Caret ranges in `[workspace.dependencies]` instead of `=` pins; the
  quarantine lives in `Cargo.lock` (`.claude/rules/dependency-quarantine.md`).
- README: absolute GitHub URLs (crates.io breaks relative ones), the
  crates.io dependency (`protocontract = "0.3"`) and `cargo install`,
  badges, the logo (`assets/logo.svg`, by raw GitHub URL).
- `keywords`, `categories`, `exclude = ["BUILD.bazel"]`, and a copy of
  `LICENSE` in both packages; `[package.metadata.docs.rs] all-features`
  for `protocontract`; the logo as rustdoc logo and favicon.
- `.bazelignore` with `target`: `cargo package` unpacks each crate into
  `target/package/` with its `BUILD.bazel`, which `//...` picked up.
- The publish step in `.claude/skills/versioning/SKILL.md`.

## Still to do

1. **First publish, by the owner** (it needs the crates.io token): log in
   on crates.io with GitHub, create an API token, `cargo login`, then run
   step 4 of the versioning skill on the release tag. Both names were free
   on 16 October 2026. A version can only be yanked, never replaced; the
   name stays with the account.
2. **Trusted Publishing** for later releases: configure it on each crate's
   crates.io settings (repository `sonalect/protocontract-buffa.rs`,
   workflow `release.yml`); it works only for a crate that exists. Then a
   `publish` job in the release workflow, after `release`, with
   `id-token: write` and the crates.io auth action pinned by commit SHA
   (quarantine), runs `cargo publish --locked` for both crates. Until
   then the owner publishes by hand.
3. **Social preview** of the GitHub repository (Settings → General →
   Social preview, 1280×640 PNG): GitHub has no API for it. The image is
   rendered from `assets/logo.svg` with the name and a tagline.

**Why:** the list was worked out once with the owner; start from it.
