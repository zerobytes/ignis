# Ignis website publication

- The user requires both Firebase Hosting (https://ignis.web.app/) and GitHub Pages (https://zerobytes.github.io/ignis/) to be updated whenever the website is published.
- Use `python3 website/deploy.py` from the engine root. Use `--prepare` before committing to refresh dist/version.json without publishing. Do not report completion until both served versions and asset hashes are verified.
- The canonical development origin is zerobytes/ignis. Commit and push the intended website release through the normal public Git history before running the paired deployment script; it verifies public source and does not create remote-only commits. Never push or merge pre-publication private history.
- Preserve shared-checkout changes. Website release authority does not authorize publishing other code, credentials, documents, workflows or artifacts.
