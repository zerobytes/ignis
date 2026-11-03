# Contributing to Ignis

Start with the [quickstart](docs/quickstart.md) and [current status](docs/status.md).
Ignis is early research software; small, reproducible contributions help most.

## Choose a path

- **Try it:** follow the quickstart and report your OS, Rust version and first
  failing command in a [bug report](https://github.com/zerobytes/ignis/issues/new?template=bug_report.yml).
- **Ask or share:** use [Discussions](https://github.com/zerobytes/ignis/discussions)
  for questions, experiments and things you made.
- **Improve a feature:** open an issue describing the use case, observable
  result and scope before starting a large change.
- **Improve onboarding:** fix a reproducible setup problem, broken link,
  confusing command or translation. Screenshots should show the actual engine.

## Development rules

The canonical development repository is **https://github.com/zerobytes/ignis**.
Clone it directly, create a branch, and submit changes through a pull request.
There is no separate private development tree or snapshot-export step. See the
[Git workflow](docs/development.md) for maintainer and website publication steps.

1. Keep offline single-player working without an account or remote service.
2. Use the shared command API. Authoring controls must be available through
   MCP before or alongside their visual editor controls.
3. Preserve state identity, atomicity, undo, persistence and replay contracts.
   Make format changes explicit; do not silently discard user work.
4. Review allocations, copies, repeated work, GPU/I/O, locks and cache
   invalidation. Report measured performance with hardware and workload.
5. Keep a pull request focused and describe behavior, validation and limits.
6. When a public capability or its integration scope changes, update the
   [feature guide](docs/features.md), [status](docs/status.md) and the relevant
   README overview. Keep research and future work explicitly labeled.

Run `cargo fmt --all -- --check` and
`cargo clippy --locked --release --all-targets -- -D warnings` for Rust changes.
Select existing tests and MCP scenarios for the affected behavior. Add tests
for material risks such as corruption, ownership, atomicity, persistence,
security or integration failures. Text and cosmetic edits do not need new tests.
GPU/headless checks do not establish native input or human artistic acceptance.

See [verification guidance](docs/quickstart.md#verification). Do not edit
bundled third-party license texts or check in local build products, private
reference images, credentials or personal data.

## License

The project is source-available under the [Ignis Engine Commercial Source
License 1.0](LICENSE), not Apache-2.0 for this distribution. Before submitting a
contribution, read the license and explicitly state in the pull request that
you offer your contribution under it. You must have authority to contribute
that work. Retain all upstream notices. Submission does not transfer copyright;
any different future relicensing of your contribution needs sufficient rights
or your separate agreement. The license identifies Guilherme Oliveira as the
recipient of applicable Game royalties; contributing does not create a promise
of revenue sharing, employment or payment.
