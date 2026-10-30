# Meteor Horizon: Solstice foundation adoption

The `solstice-integration` branch is adapting the existing MIT-licensed
[Solstice native client](https://github.com/PointOfPressure/solstice-client/tree/2b11af4dfae7b08f9faa65d6fd3f4ab7b32030d1)
as **Meteor Horizon**. This uses ordinary Horizon `.hnmod` schema 1 and upstream
Mixins, retaining internal Solstice support names where useful. It introduces no
custom loader. The candidate identifies as `meteor_client`, version
`0.1.0-solstice.2`, pinned to Horizon
`b76ae35f5b9b2d82c0e6818fc0ef93bb5277ec73` and Mixins 0.6
`cb1ad23022392d88f006cf94cbf6552a73f5df12`.

The source catalog contains **87 entries** after the reserved features are
excluded. This is an implementation/catalog count, **not 87 tested Meteor ports**
or a claim of full parity. Source adaptation and runtime verification are still
in progress; no new verified foundation download is claimed here.

The new source lives in this `foundation/` folder, a sibling to the preserved `native/`
migration.3 tree. Promotion must retain the complete existing `native/` source
and evidence plus all versioned archives. The held unfinished migration.4 stays
in its separate snapshot. The planned archive is
[packages/meteor-horizon-solstice-foundation.hnmod](../packages/meteor-horizon-solstice-foundation.hnmod); it is not yet a verified release.

## What has been checked

A separate Solstice compatibility probe on these Horizon/Mixins pins passed
official planning (23 hooks, 31.02s), apply (44.84s) and host Cargo check (47.28s).
Seven upstream unused-code warnings remained. Those results concern the probe;
they do not validate the later branded/reservation/GUI production changes.
The production R3 source is frozen and separately passed stock parsing, the official 24-hook plan/apply and host check. Six R3 binary tests pass, including re-enable/rejoin packet-policy restoration. Three Timer/NoSlow checks apply to byte-identical library code. Linked build and isolated runtime checks are pending.

Root-owned fresh-home runtime validation will record source/package/client/control
and harness hashes, observed physical Sky actions, real widget feedback and
screenshots. Count only checks actually completed against that exact candidate.
Catalog size, a passing plan or a compile do not establish gameplay parity.
Stock launcher end-to-end build/install remains unverified.

## Preserved checkpoints and contributor work

The versioned migration.3 movement package and its 101 isolated runtime checks
remain valid for their original snapshot. Its source/evidence are preserved,
not inherited as results for this foundation. The separate unfinished migration.4
RightShift GUI/HUD snapshot is held; it has no new verified UI runtime claim.
The retained Meteor core, policy tests and legacy work remain available.

AutoRespawn and BreakDelay remain reserved for a separate contributor's planned
PR. Solstice's Auto Respawn and overlapping Fast Break must stay out of active
registration/dispatch until that contribution is coordinated. Preserve the
friend's tests/source/authorship; no contributor identity is inferred. New
foundation PRs use `solstice-integration`; the planned reserved PR still uses
`hnmod-migration` until an explicit handoff.

## Source and licenses

The upstream source is pinned to commit
`2b11af4dfae7b08f9faa65d6fd3f4ab7b32030d1`. Provenance records original URLs,
committed blob and local checkout hashes, initial copies and final adaptation
notes. The local clone has CRLF-only conversion relative to upstream LF blobs;
these are recorded separately. Copying upstream source does not transfer
original authorship to this project.

Upstream declares Solstice MIT, and its notice is retained verbatim. However,
combat.rs, render.rs and world.rs explicitly describe ports from Meteor source;
util.rs describes ports of Meteor utility helpers. We have not established an
independent rewrite or that all substantive code is MIT-only. This project keeps
its GPL-3.0 distribution and Meteor notices, together with the upstream MIT and
other component notices. There is no global MIT relicense.
The project remains an unofficial, AI-assisted native port.

The upstream installer is not used: it changes real default mod files, removes
older filenames and rewrites enabled lists even before its dry-build check.
Packages use bounded deterministic source/provenance/license tooling. No game
binary, prepared Minecraft assets, account, world or private home is included.

## Publication preparation

The bundle contains the exact held R3 source package payload (42 files), SHA256
`215c62cbcc28bb7344467590873057e7dc7f2f53b916eaa7fdabd05059dc4558`.
Seven packaging tests passed on the unchanged tools; final R3 hashes passed the
frozen payload contract. Official R3 transformation/check results are recorded separately; linked build/runtime checks are pending. R1 evidence is preserved as its own source-only snapshot.

No previous Meteor package contract is broadened. This template contains no game
binary, resources, world, account, installer or compiled validator. The existing
`native/` tree and versioned packages stay unchanged when `foundation/` is added.
