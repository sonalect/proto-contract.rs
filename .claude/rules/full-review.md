# Full review

When the owner asks for a **full review**, review **all code and all
docs** in the repository. That means every handwritten source file
(`src/`, `tests/`, the plugin, BUILD, proto, golden files, scripts), and
every document (`DESIGN.md`, README, CHANGELOG, and equivalent). Not the
documents alone. Not the current diff. Not the last stage. Not the focused file.

If the workspace has more than one git root, review the repository the
owner named (or the one the current work is in). Do not fold a sibling
repo (Knowqore, for one) into the same plan unless asked.

Do **not** implement. The deliverable is a **coding plan**. Do the review
**in this session** yourself; do not hand it to a subagent or a weaker
model. A full review runs on Opus: if this session is on another model,
say so and ask the owner to switch (`/model opus`) before starting.

## Checklist

1. **Bugs** — wrong results, broken invariants, races, bad error codes, edge cases (empty, overflow, invalid input).
2. **Allocations** — `clone` / `to_owned` / `format!` / extra `Vec` on hot paths; borrow or reuse when it stays clear.
3. **Dead code** — unused, duplicated, leftover `todo!` for work already shipped.
4. **Performance** — hot loops, extra I/O or parse, lock scope, N+1, repeated descriptor walks, a bridge that hops threads without need.
5. **Docs vs code** — `DESIGN.md` and README match behaviour; no contradiction with `src/` (names, flags, codes, defaults).
6. **Spec / API** — public names and error codes match the contract; no silent defaults the spec left open.
7. **Tests** — stages marked done in `DESIGN.md` have a real test; missing negatives.
8. **Errors** — `Result` in `src/` (no `panic!` / `unwrap` / `expect` / `assert!`); `code` not replaced by `message`.
9. **Untrusted input** — paths, YAML/JSON, HTTP, env: no traversal, no secret in logs.
10. **Concurrency / isolation** — lock order, poisoning, which thread a bridge runs on, no blocking on a runtime worker.

Skip an item only when that surface is absent; say so. Do not pad the plan
with cosmetic nits.

## Plan shape

Prioritised list. Each item: severity, location, what's wrong, proposed
change. No patches until the owner asks to implement.
