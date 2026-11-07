# 17. Profiling with Tracy

**Goal:** measure how long each frame of a kit app takes and where the time
goes. It starts when you launch the app with Tracy and ends when you read the
zones in the viewer.
**Reader:** anyone looking for why an app is slow, or checking that a change
speeds it up.
**Inputs:** an app that calls `barrier_kit::init_tracing()` and the
`barrier-kit/tracy` feature.
**Outputs:** a trace in the [Tracy](https://github.com/wolfpld/tracy) viewer
with one frame per `draw` and its zones.

Español: [17-perfilado.md](17-perfilado.md).

## Steps

1. Enter the dev shell: `nix develop`. It provides the `tracy` viewer.
2. Launch the app in release with the Tracy layer:

   ```sh
   cargo run --release --features barrier-kit/tracy -p <app>
   ```

3. Open the viewer in another dev shell terminal and connect to the app that
   shows up in the list:

   ```sh
   tracy
   ```

4. Use the app. The client only keeps data while the viewer is connected.
5. Read the zones of each frame:

   | Zone | Measures | Code |
   |---|---|---|
   | `draw` | A surface's whole frame | `platform/surface.rs` |
   | `acquire` | The wait for the swapchain image | `platform/surface.rs` |
   | `layout` | Layout and the `on_sized` reactions | `platform/surface.rs` |
   | `render` | Recording the Skia commands | `platform/surface.rs` |
   | `glass` | The progressive glass, one pass per layer | `glass.rs` |
   | `material` | The blurred media material | `glass.rs` |
   | `present` | Submission to the GPU and presentation | `platform/surface.rs` |

## How to read it

- `render` measures CPU. GPU work shows up as waiting in `acquire` and
  `present`.
- Freya 0.4.3 repaints the whole tree every frame. A still window must not
  produce new frames.
- Each popup has its own `draw`, so it adds frames to the trace.

## A new app

1. Call `barrier_kit::init_tracing()` at the start of `main`. It honours
   `RUST_LOG` like `tracing_subscriber::fmt::init()`.
2. Build with `--features barrier-kit/tracy`. Without the feature, the zones are
   `trace` level and cost nothing under the default filter.

## Tracy version

The viewer and the client must speak the same protocol. The flake ships Tracy
0.13.1, and `Cargo.lock` pins `tracy-client` 0.18.4 and `tracy-client-sys`
0.28.0, its matching pair. If you bump one, bump the other in the same commit.
