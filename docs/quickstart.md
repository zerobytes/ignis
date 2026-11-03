# Quickstart

## Requirements

- Rust 1.88 or newer, including Cargo. Use a current stable toolchain if a locked
  dependency requires a newer compiler.
- Native compiler/linker tools. On macOS, install Xcode Command Line Tools
  (`xcode-select --install`) if they are missing.
- A compatible graphics driver for the wgpu backend when opening a window or
  capturing pixels. Plain inspection and simulation do not initialize a GPU.
- Python 3 for optional verification and authoring scripts. Individual research
  scripts may have additional dependencies; they are not needed for the first run.

Active desktop development and native acceptance currently center on macOS
Apple Silicon/Metal. Linux and Windows use the same Rust sources, but are not
promised as accepted desktop targets. On Linux, default device audio requires
ALSA development headers (`libasound2-dev` on Debian/Ubuntu); window integration
may require additional distro-specific development libraries. A build without
physical audio uses `cargo build --locked --release --no-default-features`.
This disables device output, not the internal sound model.

CI uses Rust 1.97.1 for a reproducible lint baseline, matching the development
toolchain used for this source preview. Newer compilers can introduce new lints.

## Build and inspect

```sh
git clone https://github.com/zerobytes/ignis.git
cd ignis
cargo build --locked --release
./target/release/ignis --help
./target/release/ignis --root "$PWD" \
  --scene examples/stealth-lab.scene.json \
  --command '{"command":{"op":"inspect"}}'
```

Inspection returns JSON. On Windows, use `target/release/ignis.exe`, your
checkout's absolute path and your shell's JSON quoting rules.

## Open the editor or play

```sh
./target/release/ignis --root "$PWD" --editor --scene examples/stealth-lab.scene.json
```

Use `--window` instead of `--editor` for the playable view. **P** starts/pauses,
**WASD/arrows** moves and **R** resets. The scene starts paused. Try
`examples/npc-ground-navigation.scene.json` for the ramp/controller experiment.
Each demo's document explains its controls and intended behavior.

## MCP walkthrough

Configure a local stdio MCP client as shown in the [README](../README.md#connect-an-agent).
Use absolute paths, not a literal `$PWD` inside JSON. Add `--editor` for a shared
visual session, or leave it out to run headlessly. Two processes are two
independent sessions. There is no HTTP MCP endpoint in this setup.

After the MCP handshake, `tools/list` exposes `engine_command` with its generated
JSON Schema. Send these as the tool's arguments in an expendable scene:

```json
{"command":{"op":"inspect"}}
```

```json
{"command":{"op":"generate","recipe":{"width":16,"depth":14,"seed":42,"obstacles":4}}}
```

```json
{"command":{"op":"step","ticks":120}}
```

```json
{"command":{"op":"capture","path":"artifacts/view.png","width":1280,"height":900}}
```

`generate` replaces the scene: inspect first and save work you need to keep.
Use `expected_revision` for edits that must reject stale state. `capture`
returns an image block with actual rendered PNG pixels and state metadata.
The renderer initializes on demand. File paths are relative to `--root` unless
an operation explicitly selects a project folder. Logs go to stderr; stdout is
newline-delimited JSON-RPC. [Detailed contract](commands.md).

## Verification

Select checks appropriate to your change. For example:

```sh
cargo fmt --all -- --check
cargo test --locked --release --test local_transactions
cargo clippy --locked --release --all-targets -- -D warnings
python3 scripts/run_verifications.py --binary target/release/ignis --only local_transactions
```

The MCP runner isolates scenes and outputs under `artifacts/mcp-validation` and
uses the supplied binary; it does not build it. Run it from the repository
root. Offscreen captures require a working GPU backend even with no window.
The aggregate runner is a selected regression collection, not proof of every
research script or platform. Broad tests can be expensive; check each scenario's
requirements before running it. [Contribution guidelines](../CONTRIBUTING.md).

## Troubleshooting

| Symptom | Next step |
| --- | --- |
| Compiler too old | Update Rust and retry with `--locked`; do not regenerate the lockfile just to suppress a setup error. |
| Audio headers missing | Install the native audio development package or build with `--no-default-features`. |
| GPU adapter unavailable | Update drivers; use `inspect` to separate simulation setup from graphics setup. A headless machine still needs a GPU/software adapter for capture. |
| MCP client cannot start | Confirm both absolute paths, executable permissions and that the client supports local stdio servers. |
| Scene or asset missing | Keep `--root` at the checkout root for included examples. Private reference inputs and generated artifacts are intentionally not bundled. |
| Replay rejected after rebuilding | Replays are scoped to their originating build. Use the documented scene or portable-save formats for the applicable persistence use case. |

For an unresolved problem, file a [bug report](https://github.com/zerobytes/ignis/issues/new?template=bug_report.yml)
with the revision, OS, Rust version and first failing command. Remove secrets
and private local paths from logs.
