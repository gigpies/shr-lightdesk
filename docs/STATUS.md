# Status

[Using the lighting desk](USING.md) · [Capabilities](CAPABILITIES.md) · [GigPies status](https://github.com/gigpies/gigpies/blob/main/docs/STATUS.md)

**Software-accepted against the real SHR Lux engine with null lighting output.
Physical DMX output, controllers and displays are not yet qualified.**

| Area | Works in software | Still open |
|---|---|---|
| Looks and cues | Programmer, Hold, cues, palettes, playback and release previews through Lux writer leases | Fixture personalities/patch authoring wait for Lux |
| Scenes | Owner-backed scene maps with nonmutating previews and reviewed intensity/RGB scopes | Physical output (Lux LX-06) |
| Music-following | Calibration, bounded AUTO, seven-feed musical context: key/chord/Camelot, direction, pauses, tentative sections | Rehearsal accuracy (GigPies GP-H3); no beat/downbeat |
| Controllers | Semantic eight-pad menu, value entry; learned pad profile (`--controller-profile`) with read-only MIDI acquisition | Learned MiniLab profile and hardware checks (GigPies GP-H2) |
| Surfaces | Ratatui terminal (shared panel 0.3.3), optional native window, CPU offscreen checks | Physical display acceptance |

Exact contracts and boundaries are in [capabilities](CAPABILITIES.md). Shared
integration tasks live in the
[GigPies integration plan](../../gigpies/docs/development/MODULE_IMPLEMENTATION_PLAN.md).
Dated validation records up to 2026-11-24 are in [the evidence archive](archive/evidence-to-2026-11-24.md).
