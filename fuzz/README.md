# kime-fuzz

cargo-fuzz targets for the engine and its parsers. Nightly-only, so the
crate is excluded from the root workspace and carries its own lockfile.

## Targets

| target | what it checks |
|---|---|
| `engine_key_stream` | arbitrary key/op sequences against `InputEngine`: no panic, and the `HAS_COMMIT`/`HAS_PREEDIT` flags stay in sync with the commit and preedit buffers |
| `engine_diff_libhangul` | the same dubeolsik syllables through kime and through libhangul (the composition engine behind ibus-hangul and fcitx5-hangul) must produce the same text — the engine-level guard for last-syllable (끝글자) bugs |
| `frontend_commit_protocol` | a Rust mirror of the GTK immodule's commit/emit/reset protocol driving the real `InputEngine` against a hostile client that re-enters `reset()` from its commit and preedit handlers — every committed character must reach the client exactly once, in order |
| `layout_yaml` | `Layout::load_from` never panics on arbitrary input |
| `config_yaml` | `RawConfig` deserialization never panics on arbitrary input |

`frontend_commit_protocol` fuzzes a hand transcription
(`src/frontend.rs`) of `src/frontends/gtk3/src/immodule.c`
(`process_input_result`, `kime_reset`, `commit`, `update_preedit`) — the
protocol layer where the [#562] Firefox reset-in-commit-handler double
commit lived, fixed in [#799] by snapshotting and clearing the engine
buffer before emitting (the Qt frontend uses the same ordering). The
hostile client may synchronously call `reset()` from any commit delivery
or preedit signal, at any depth. Because it is a transcription and not the
C code, it can drift: whenever the commit/reset ordering in `immodule.c`
or `input_context.cc` changes, `src/frontend.rs` must be re-transcribed by
hand. Triage rule: reproduce any finding with the e2e gtk/qt reset probes
(`tests/e2e`: `g3_06`/`g3_07`, `q06`/`q07`) against the REAL frontends
before filing.

[#562]: https://github.com/Riey/kime/issues/562
[#799]: https://github.com/Riey/kime/pull/799

`engine_diff_libhangul` generates whole syllables (onset, vowel, optional
second vowel, optional coda) rather than free key sequences. kime and
libhangul genuinely disagree on consonant runs — kime commits a lone jamo
where libhangul keeps building a cluster in its preedit — and that
difference is reachable so easily that a free-form generator never gets
past it to the commit-timing behaviour the target exists to check.

## Running

```sh
nix develop .#fuzz     # nightly rustc, cargo-fuzz and libhangul
cd fuzz
cargo fuzz run engine_key_stream -- -max_total_time=600
```

Without nix: `rustup toolchain install nightly`, `cargo install
cargo-fuzz`, and install libhangul (every target links the crate, so its
`.pc` file must be on `PKG_CONFIG_PATH` even for the parser targets).

Starting inputs are assembled into `corpus/<target>/` by
`prepare-corpus.sh`, which CI runs too:

```sh
./prepare-corpus.sh layout_yaml
cargo fuzz run layout_yaml
```

It never points cargo-fuzz at tracked files — cargo-fuzz writes what it
discovers into the corpus directory, which would bury the inputs in
thousands of generated files. What kime already ships is copied in from
where it lives (`res/default_config.yaml` for `config_yaml`, the layouts
under `src/engine/backends/hangul/data/` for `layout_yaml`), so
`seeds/<target>/` holds only inputs that exist nowhere else.

CI (`.github/workflows/fuzz.yaml`) runs every target daily at 03:00 KST,
persisting `fuzz/corpus` across runs through actions/cache, and uploads
the crashing inputs when a run fails. `workflow_dispatch` takes a target
list and a per-target time budget.

## Triage — a crash artifact appeared

1. Reproduce: `cargo fuzz run <target> artifacts/<target>/crash-<hash>`
2. Minimize: `cargo fuzz tmin <target> artifacts/<target>/crash-<hash>`
3. File an issue describing the minimized reproduction.
4. Create `bases/fuzz-<issue#>` off develop and commit the minimized input
   as a plain failing `#[test]` in the crate that owns the bug — red until
   the fix merges, the same policy the e2e suite follows
   (`tests/e2e/README.md`).
5. Stack the fix PR on that branch so its CI run shows the red→green flip.

For an `engine_diff_libhangul` divergence, first decide by hand which side
is right. kime wrong is a product bug — steps above. If both are
defensible, the difference belongs in `normalize()` (`src/diff.rs`) with a
comment naming the finding it came from; that function exists to absorb
representation differences, not to quiet real ones.
