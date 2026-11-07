# 19. Profiling

**Goal:** measure how long each frame of a BarrierKit app takes and which
step uses the time.
**Scope:** from building the app with the `tracy` feature to reading the
zones of a frame in the [Tracy](https://github.com/wolfpld/tracy) viewer.
What a frame does is in [18. Platform](18-platform.md#the-frame).
**Reader:** whoever looks for why an app is slow, or checks that a change
makes it faster.
**Input:** an app whose `main` calls `bk::init_tracing()`.
**Output:** a Tracy trace with one frame per draw and its zones.

## Steps

1. Enter the dev shell: `nix develop`. It provides the `tracy` viewer.
2. Build and run the app in release with the feature:

   ```sh
   cargo run --release --features bk/tracy -p <app>
   ```

3. In another dev-shell terminal, start the viewer and connect to the app in
   its list:

   ```sh
   tracy
   ```

4. Use the app. The client records only while a viewer is connected.
5. Read the zones of each frame ([table](#zones)).

## `init_tracing`

`bk::init_tracing()` installs the global `tracing` subscriber. Call
it once, at the start of `main`.

| Layer | Filter | Present |
|---|---|---|
| Text log on stderr (`tracing_subscriber::fmt`) | `RUST_LOG` as `target=level` directives; `info` when unset or invalid | Always |
| `tracing_tracy::TracyLayer` | None: it gets every span | With the `tracy` feature |

`RUST_LOG` is parsed as a `tracing_subscriber::filter::Targets` list, such as
`RUST_LOG=info,bk::resize=debug`. Span-field filters are not
supported.

The zones are `trace` spans. Without the `tracy` feature and with the default
filter, no layer wants them, so they cost almost nothing.

## Zones

| Zone | Measures | Code |
|---|---|---|
| `draw` | One surface's whole frame | `platform/surface.rs` |
| `acquire` | The wait for the next swapchain image | `platform/surface.rs` |
| `layout` | Layout and the `on_sized` reactions | `platform/surface.rs` |
| `render` | Recording the Skia commands for the whole tree | `platform/surface.rs` |
| `glass` | The progressive glass, one pass per layer | `glass.rs` |
| `material` | The blurred material | `glass.rs` |
| `present` | Submitting to the GPU and presenting | `platform/surface.rs` |

After `present`, the runner sends a Tracy frame mark when a viewer is
connected.

## How to read a frame

- `render` is CPU time. GPU time shows up as waiting in `acquire` and
  `present`.
- Each draw records the whole tree, so `render` grows with the tree, not with
  the change.
- A still window draws nothing. Frames in a still window mean a state or an
  animation keeps asking for redraws.
- Every surface has its own `draw`: a popup or a second window adds frame
  marks to the same trace.
- `glass` and `material` appear inside `render` only when the frame contains
  those elements.

For timings without the viewer, the `bk::resize` target logs each
draw start and the present time (`swap_us`) per surface:

```sh
RUST_LOG=info,bk::resize=debug cargo run --release -p <app>
```

## Tracy version

The viewer and the client must speak the same protocol version.

| Side | Version | Where it is fixed |
|---|---|---|
| Viewer | Tracy 0.13.1 | nixpkgs in `flake.lock` (`tracy`) |
| Client | `tracy-client` 0.18.4 + `tracy-client-sys` 0.28.0 (Tracy 0.13.1) | `Cargo.lock` |

If you update one side, update the other in the same commit.

Next: [20. Glossary](20-glossary.md).
