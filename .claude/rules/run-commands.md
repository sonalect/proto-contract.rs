# Run commands from this repository

Every `bazel`, `cargo`, and `buf` command for Contratto **must** run in
this clone's root: the directory with `MODULE.bazel` and the workspace
`Cargo.toml`. The Bash tool's working directory persists between calls and
may have drifted (another repository may be the session's primary
directory), so `cd` to the root by absolute path in the same command when
unsure. Never run them from a sibling project, a scratch or tmp directory,
or a subdirectory.

## Generate

The test protos under `proto/` are compiled by the plugin built from this
tree into golden files. After changing a test proto or the plugin's
output, regenerate the golden files with the build's generate target
before tests, and read the diff: a changed golden file is a
consumer-visible change (`consumers.md`). Never edit a golden file by
hand.

## Fast loop and gate

- `cargo test --workspace` and
  `cargo clippy --workspace --all-targets --all-features`: the fast loop.
- `bazel test //... --keep_going --test_output=errors`: the gate before a
  commit (`commit-after-tests.md`). It also runs `//bazel:lint`
  (buildifier) and `//bazel:markdown` (markdownlint over every `*.md`,
  `.claude/` included). `bazel run //bazel:format` fixes BUILD formatting.

## No sandbox

Do not run `bazel`, `buf`, or `cargo` inside a command sandbox. Bazelisk
needs the network (`.bazelversion`, BCR, crate_universe), the cache lives
under `~/.cache/bazel`, `buf generate` needs the Buf Schema Registry, and
generate writes into the source tree. If Bash sandboxing is on, run these
with `dangerouslyDisableSandbox: true`.
