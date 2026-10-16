# bazel_utils catalog entry for the plugin

## State

- `../bazel_utils` (`protoc/plugins/`) catalogs the plugin under its old
  name, `protoc-gen-contract-rust`, versions v0.1.0, v0.2.0, and v0.2.1,
  with release URLs and README links on `sonalect/proto-contract.rs`.
  GitHub redirects that URL to `sonalect/protocontract-buffa.rs`, so the
  old entry keeps working.
- The owner renamed the project on 16 October 2026: repository
  `sonalect/protocontract-buffa.rs`, plugin and binary
  `protoc-gen-protocontract-buffa`, Bazel module `protocontract_buffa`.
  The release binaries take the new name from the first release after the
  rename, so the old catalog entry cannot list that release.
- Knowqore (`../knowqore`) still depends on `sonalect/proto-contract.rs`
  and the old plugin name.

## How to finish

1. Release the renamed plugin here (`.claude/skills/versioning/SKILL.md`).
2. In `../bazel_utils`: add `protoc/plugins/protoc-gen-protocontract-buffa/`
   with the sha256 values from that Release's `checksums-sha256.txt`, and
   register it everywhere its `CLAUDE.md` lists ("Adding a protoc
   plugin"). Keep or drop the old `protoc-gen-contract-rust` entry; that
   is the owner's call.
3. Knowqore takes the new plugin name, the new repository URL, and the new
   tag.

**Why:** a consumer pins the plugin by its catalog name, and the binary
names in a Release are the catalog's download URLs.
