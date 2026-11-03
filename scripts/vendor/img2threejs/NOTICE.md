# img2threejs camera-fitting subset

Source: https://github.com/img2threejs/img2threejs
Revision: 3e7099e1b7fe93e187e06db4bdbe1e1906c21a28
Upstream paths: forge/stage1_intake/camera_fitting_{types,math,solver}.py
License: Apache-2.0, included in LICENSE. Upstream repository contributors retain
copyright in their work. No upstream NOTICE file was present in this revision.

Changes on 2026-11-01: module imports made package-relative and SPDX/provenance
headers added. Numerical algorithm and validation are otherwise unchanged.
Only these three modules are vendored; no installer, model, image or optional
integration is included. The Ignis wrapper and review ledger are separate code.

Also includes tapered_sweep.ts, extracted from the emitted string literals of
forge/stage3_build/generate_threejs_factory.py. Added module import/export and
made the last UV-seam sample use exactly the first angle to avoid a tiny gap.
No other numerical changes to this helper. Copyright 2026 hoainho.
