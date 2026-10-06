# A release tag carries the release notes

A version tag starts `.github/workflows/release.yml`, and its message
becomes the GitHub Release: the subject is the title, the body the notes.
The workflow refuses a tag that is not annotated, does not point at a
commit on `main`, or has no body. The procedure around it is in
`.claude/skills/versioning/SKILL.md`.

## When

Only when the owner asked to release, after the version commit is on
`main` and pushed. Never tag another branch, never move or recreate a tag
that is pushed, never a lightweight tag.

## The message

- **Subject:** `vX.Y.Z: <what this release gives, in one line>`.
- **Body:** the release notes, for someone deciding whether to upgrade.
  Markdown, English. The `CHANGELOG.md` section of the version, in the
  same `### Added` / `### Changed` / `### Fixed` / `### Removed` groups,
  plus, for a breaking change, what a consumer changes (renamed items,
  new signatures, parameters). No file lists, no commit hashes.

Write it to a file and tag with `--cleanup=verbatim`. Without it git
strips every line that starts with `#`, so every Markdown heading of the
notes is lost. Never let git open an editor.

```bash
notes="$(mktemp)"
cat > "$notes" <<'NOTES'
v0.1.0: sync and async traits for protobuf services

### Added

- `protoc-gen-contract-rust` emits `<Service>Sync`, `<Service>Async`, and
  `Dyn<Service>Async` for every service, streaming methods included.
NOTES
git tag -a v0.1.0 -F "$notes" --cleanup=verbatim
git tag -l --format='%(contents:subject)%0a%0a%(contents:body)' v0.1.0
git push origin v0.1.0
rm -f "$notes"
```

Read the tag back (the `git tag -l` line) before pushing it: the pushed
message is the published release.

```text
# BAD — the headings are gone, and the notes say nothing
git tag -a v0.1.0 -m "v0.1.0" -m "### Added ..."
# GOOD — file, --cleanup=verbatim, read back, then push
```
