# Project notes

Project state, deferred work, measurements, and references that must
outlive one machine (see `.claude/rules/project-notes.md`). Read this index
before planning; open a note when its topic comes up.

| Note | What it holds |
| - | - |
| [bazel-utils-catalog.md](bazel-utils-catalog.md) | The plugin's bazel_utils catalog entry: catalogued under the old name `protoc-gen-contract-rust`; the steps for the renamed plugin in bazel_utils and Knowqore |

## References

- Design and decisions: `DESIGN.md` at the repository root.
- Anchor consumer: Knowqore, `github.com/sonalect/knowqore`, usually
  cloned as `../knowqore` (`.claude/rules/consumers.md`).
- Plugin catalog for Bazel: `github.com/sonalect/bazel_utils`,
  `protoc/plugins/` (one `registry.bzl` per plugin).
