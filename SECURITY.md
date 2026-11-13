<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Security Policy

## Supported versions

Security fixes are developed against the latest commit on `main`. Until AureaOS publishes a stable release, no older revision is considered a supported security-maintenance line.

## Reporting a vulnerability

Please do not publish exploitable details in a public issue or pull request.

Use [GitHub's private vulnerability reporting](https://github.com/AleessandroCeccarelli/AureaOs/security/advisories/new) to contact the maintainers privately. Include, where possible:

- the affected component, file, and commit;
- the security boundary and attacker capabilities required;
- clear reproduction steps or a minimal proof of concept;
- the expected impact and any known mitigations.

Do not include real credentials, private keys, personal data, or production secrets in a report.

The maintainers will acknowledge the report, investigate its reproducibility and impact, and coordinate a fix and disclosure timeline with the reporter. Response times are best-effort while the project is in active development.

## Scope and current maturity

AureaOS is under active development and is not yet a production-hardened operating system. A passing test suite or dependency audit is not a security certification and does not establish that unfinished compositor, graphics, session, installation, or system-service paths are safe for production use.
