# CLAUDE.md

## CHANGELOG (docs/CHANGELOG.md)

- A changelog entry describes the net change a PR makes. Bugs that were
  introduced and fixed within the same PR (review follow-ups, intra-PR
  reverts) must not get their own entries — update the PR's single entry to
  describe the final behavior instead.
- Changes to something an earlier PR introduced (including reverts, cf. the
  [#754] entry reverting [#719]) are real changes and are mentioned normally.

## Pull requests

- Label every PR using the repo's existing labels: pick the matching area
  label(s) — `T-Engine`, `T-GTK`, `T-Qt`, `T-Wayland`, `T-XIM`, `T-Build`,
  `T-Tool`, `T-Document` — and add `bug` for bugfixes, `Breaking` for
  breaking changes, `Package` for packaging work. Don't create new labels.
