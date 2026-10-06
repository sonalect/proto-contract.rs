# Consumers outside this repository

Contract is a general-purpose generator and runtime for any Rust project
that uses buffa, other people's included. Knowqore
(`github.com/sonalect/knowqore`, usually cloned next to this repository as
`../knowqore`) is its **anchor consumer**: the first user, the motivation
and the source of requirements for current work. It is not the only one,
and Contract is not built for it alone.

## Requirements from the anchor consumer

- Generalize a Knowqore need into a capability any consumer could use. The
  generated code, the runtime crate, names, defaults, and docs never
  mention Knowqore or its domain.
- A Knowqore example may motivate a decision in `DESIGN.md`; the normative
  text states the general rule.
- When a Knowqore need and a clean general design disagree, say so to the
  owner instead of bending the generator toward one caller.

## When a change can break a consumer

The generated code is the product. A change to a generated name, a method
signature, the module layout, a plugin parameter or its default, a runtime
type, the `Status` ↔ `ConnectError` mapping, or the thread a bridge runs
on is consumer-visible — for every consumer, known or not. A changed
golden file is such a change by definition.

Before calling such a change done:

1. Write the effect into `CHANGELOG.md` under `Changed` / `Removed`, in
   words any caller can act on.
2. Bump per `.claude/skills/versioning/SKILL.md` at release: breaking →
   minor while major is 0.
3. Once Knowqore uses Contract, check it as a real-world sample:
   `git grep -n contract ../knowqore` (or ask `scout`), and tell the
   owner which of its call sites change. Knowqore passing is a sample, not
   proof that other consumers are safe.

Contract sits next to two generators it does not own. The Rust paths it
emits must match `protoc-gen-buffa`, and the Connect adapters must match
`protoc-gen-connect-rust`, at the versions this repository pins. A bump of
either is tested against the golden files before it lands.

Do not edit a consumer's repository from this session unless the owner
asks. `readiness.md` § "A plan that crosses repositories" applies.
