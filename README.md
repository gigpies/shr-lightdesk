# SHR Lightdesk

**The lighting desk for GigPies.** Select fixtures, build looks, record cues and
review transitions. Manual operation comes first; ASSIST and AUTO are optional.

[**Using the desk**](docs/USING.md) · [**GigPies guide**](https://github.com/gigpies/gigpies/blob/main/docs/README.md) · [Engineering](docs/ENGINEERING.md)

## Work with looks and scenes

- Edit intensity, colour and position where the fixture supports them.
- Record and play cues; Hold a look or review its release.
- Preview authored scene maps and optional music-following changes.
- Use the terminal console or optional native window.

SHR Lux runs the lighting engine. **Current integrated operation has no physical
DMX output.** Screen previews, software controls and recovery are implemented;
physical lights, displays and controllers still need verification.

## Try the simulator

From this checkout, with its pinned Rust toolchain:

```sh
CARGO_INCREMENTAL=0 cargo run --locked -- simulate
```

Try `select 11 12`, `set INT 45`, `cue record 1`, `go 1 1`, then `help`.
The simulator keeps its show only until exit and controls no lights.
[Build instructions](docs/DEVELOPMENT.md).

For a configured GigPies system, start with the [desk controls](docs/USING.md).

[Software status](docs/STATUS.md) · [MIT license](LICENSE) · [Font notices](THIRD_PARTY.md)
