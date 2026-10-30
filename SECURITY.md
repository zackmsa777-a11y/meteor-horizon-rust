# Security policy — Meteor mod for Horizon

This policy covers this repository's native Meteor mod, standard `.hnmod` migration,
legacy source-patch `.hmod` prototype packaging, and the host/launcher
integration included to run it. Reports are handled by this repository's maintainer,
[@zackmsa777-a11y](https://github.com/zackmsa777-a11y).

## Report a vulnerability privately

Use [Report a vulnerability](https://github.com/zackmsa777-a11y/meteor-horizon-rust/security/advisories/new)
in this repository's Security tab. Private vulnerability reporting is enabled.
Please keep exploit details and sensitive data out of public issues.

Include the affected branch, commit or package hash, operating system, impact and minimal
reproduction steps. Use generated files or a disposable test world where possible. Remove
tokens, account information and personal data from logs before attaching them.

Ordinary crashes, missing modules and feature requests belong in this repository's
[issue tracker](https://github.com/zackmsa777-a11y/meteor-horizon-rust/issues).

## Development status and scope

`main` identifies the verified development checkpoint; `next-source` is work in progress.
There is no published support lifetime or response-time guarantee. A runtime check applies
only to the exact binary and scope recorded in its evidence.

The legacy source-patch `.hmod` is not a content mod; it requires the prototype's custom build path.
The intended `.hnmod` distribution uses Horizon's native-mod tooling and is still being migrated.
Both native build paths compile Rust source into code running in the host process.
That code runs with the host process's access to the computer. The installer requires explicit
native-code trust and checks the pinned base and source hashes; these checks establish package
integrity, not a sandbox or a security audit. Review source before trusting a package.

This repository currently has GitHub Actions disabled. It does not operate Horizon's signed
release pipeline, Discord report bot, automated merging or launcher update service. Inherited
host workflows and signing code are source context, not evidence that those services are
configured for this mod.

For a vulnerability specific to the original Horizon project, use its own current reporting
policy. When the affected component is unclear, report privately here first so its scope can
be investigated. Do not assume upstream maintainers receive reports filed in this repository.
