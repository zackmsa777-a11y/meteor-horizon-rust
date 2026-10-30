# LowHop at zero height

The tested AutoJump adapter follows the retained policy: LowHop writes only the
requested Y velocity. It does not force ground flags or assign X/Z velocity.
See the [core policy](../bundle/mixins/meteor_client/core/src/modules/auto_jump.rs)
and [adapter](../bundle/mixins/meteor_client/runtime.rs).

At height zero, the verified run observed no elevation or positive Y impulse.
The native ground flag was false in 8 of 18 stationary samples and 15 of 26
walking samples, while rise and lowest offset stayed at zero. Native collision
code recomputes ground from a downward collision; a zero-Y move can clear that
flag, and the following gravity-driven downward move can restore it at the floor.

The pinned host code is [entity_move.rs](https://github.com/horizon-rust/horizon/blob/b76ae35f5b9b2d82c0e6818fc0ef93bb5277ec73/crates/horizon-world/src/entity_move.rs#L175-L181).
[living_air.rs](https://github.com/horizon-rust/horizon/blob/b76ae35f5b9b2d82c0e6818fc0ef93bb5277ec73/crates/horizon-world/src/living_air.rs#L98-L162)
also uses ground state for acceleration and horizontal friction. A vertical-only
adapter write therefore does not promise identical horizontal speed after native
simulation.

The sampled steady horizontal speeds were approximately 0.117859 with AutoJump
disabled and 0.132226 during zero-height LowHop walking. Continued finite movement
was verified; equal speed was not asserted. These sampled values and counts are
fixture observations, not universal physics constants or exact apex guarantees.

Earlier harness assumptions about permanently true ground flags and equal speed
were corrected without changing the mod package or client. The accepted result
is the [50-check frozen run](runtime-autojump.json); no production ground-state
override was introduced.
