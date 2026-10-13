# Security Policy

## Supported Versions

Only the latest release of `glassmon` receives active security updates.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1.0 | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in `glassmon`, please do **not** disclose it in a public issue or discussion.

Instead, please report it confidentially using **[GitHub Private Vulnerability Reporting](https://github.com/7assanyahya/glassmon/security/advisories/new)**.

If GitHub Advisories is inaccessible, you may reach maintainers directly at:
- **Email:** `security@glassmon.dev`

Please include:
1. Steps to reproduce the vulnerability (including terminal environment, OS, architecture).
2. Proof of concept command or exploit.
3. Impact assessment.

You will receive an initial response within 48 hours. If the vulnerability is accepted, we will collaborate on a private advisory and release a patched version before public disclosure.

## Privacy & Local Execution Guarantee

`glassmon` is designed with strict privacy principles:
- **100% Local Execution:** `glassmon` never makes network calls, collects telemetry, or phones home.
- **No Analytics:** No tracking, usage metrics, or error telemetry are ever transmitted.
- **Sensors & Access:** The application queries local hardware interfaces (macOS `sysctl` / `system_profiler`, Linux `/sys` and `/proc`). It requests no root/sudo privileges by default; operations that require privilege (like raw SMART reads) will display `needs sudo` or `n/a` without attempting elevation.
- **Offline Safe:** `glassmon` functions fully in air-gapped environments.
