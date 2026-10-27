# Using the lighting desk

[Project home](../README.md) · [GigPies guide](https://github.com/gigpies/gigpies/blob/main/docs/README.md) · [Engineering](ENGINEERING.md)

Current integrated operation controls software lighting state. **Physical DMX
output is not connected.** The simulator and a connected Lux session are separate.

## Controls without computer shortcuts

The live desk is intended for its assigned MIDI controller. Computer-keyboard
shortcuts are optional fallback controls for the same actions.

The terminal footer provides **Previous, Next, Choose, Cancel, Character−,
Character+, Add and Delete**. Previous/Next opens the labelled action menu;
Choose acts on a full press and release. The menu includes connection/control,
page navigation, value editors and review actions. Select an editor, choose and
add characters, then accept/review the value through the menu. Inspect the actual
review before choosing Confirm; opening the menu does not count as reading it.
The footer is also clickable. F10 is the optional keyboard menu shortcut.

This software path accepts explicitly bound controller messages; it does not
open a MIDI device or install a preset. Physical input acquisition, learned
bindings and device acceptance remain unfinished under
[GP-H2](https://github.com/gigpies/gigpies/blob/main/docs/development/MODULE_IMPLEMENTATION_PLAN.md#gp-h2--physical-dual-console-operation).
The key references below document the optional computer-keyboard fallback.

## Try a look in the simulator

```sh
CARGO_INCREMENTAL=0 cargo run --locked -- simulate
```

```text
select 11 12
set INT 45
set RED 15
cue record 1
go 1 1
help
```

The show remains in memory only until exit. These simulator commands do not
connect to Lux or control lights.

## Open a configured desk

After [building](DEVELOPMENT.md), use the endpoint and show identity supplied
by your GigPies setup:

```text
shr-lightdesk terminal --socket-path ENDPOINT --show-id SHOW_UUID --epoch EPOCH
```

Use at least 160×45 terminal cells. Startup is read-only. Control also requires
the configured desk role and an explicit grant. The optional native window uses
`native` instead of `terminal`. [Connection setup](reference/CLIENT.md).

## Build and play a look

| Action | Key |
|---|---|
| Focus a fixture / select it | Left/Right / Space |
| Focus an attribute / enter a value | Up/Down / E |
| Record a cue / choose cue playback | R / G |
| Review Clear-to-Hold / release preview | C / P |
| Review blackout / operating mode | B / M |
| Select page / request control | F1–F5 / F6 |
| Show help / cancel | F12 / Esc |

Enter reviews the draft. Release Enter, inspect every review page, then press
Enter again to confirm. Recording a cue does not play it. Clear-to-Hold retains
the current look; release previews where it will return before committing.

## Scenes and music-following

**F7** opens authored scenes. **S** edits a scene; **V** requests its preview.
Use [scene controls](SCENE_MAP.md) for map import/export and permitted changes.

Where configured analysis is available, **K/L** review calibration start/finish,
**A** reviews AUTO entry, **T** edits the selected fixtures' intensity cap and
duration, and **X** reviews returning to ASSIST. Grants expire; they do not renew
automatically. Leaving AUTO can retain the last look.

Health shows the input analysis and what currently controls each value.
Harmony and section labels are estimates. A preview or acknowledged change is
not evidence that a physical light changed.

## Recover without losing your place

Stale values are last-known values. Reconnect, read the current look and request
control again before applying a new review. Drafts may remain visible after
focus or connection loss, but old confirmations cannot be reused.
Closing the desk does not force blackout or clear Lux's look.
