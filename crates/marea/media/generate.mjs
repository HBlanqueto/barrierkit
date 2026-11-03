// Renders the mockup's scenes into the H.264 test videos and stills Marea plays.
// Usage: node generate.mjs   (needs Node, Playwright with Chromium, and ffmpeg with libx264)

import { spawn } from "node:child_process";
import { readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";

const here = dirname(fileURLToPath(import.meta.url));
const { chromium } = createRequire(import.meta.url)("playwright");

const WIDTH = 1280, HEIGHT = 720, FPS = 24, SECONDS = 10, STILL_AT = 2.5, BATCH = 12;
const CATALOG = {
  orilla: ["waves", 205], neon: ["neon", 300], pan: ["heat", 40], raras: ["flicker", 0],
  carton: ["rain", 355], tronos: ["snow", 210], oso: ["steam", 25], separ: ["grid", 175],
  oficina: ["city", 215], lazo: ["stadium", 120], espejo: ["glitch", 0], hipotecas: ["city", 40],
  pixel: ["pixel", 205], lobos: ["fog", 260], cometas: ["waves", 160], altamar: ["waves", 195],
};

const encode = (out) => {
  const ffmpeg = spawn("ffmpeg", ["-y", "-loglevel", "error", "-f", "image2pipe", "-framerate", String(FPS), "-c:v", "mjpeg", "-i", "-",
    "-c:v", "libx264", "-preset", "slow", "-crf", "30", "-pix_fmt", "yuv420p", "-profile:v", "high", "-movflags", "+faststart", "-g", String(FPS * 2), out],
    { stdio: ["pipe", "inherit", "inherit"] });
  return { write: (buf) => new Promise((ok) => ffmpeg.stdin.write(buf) ? ok() : ffmpeg.stdin.once("drain", ok)),
           end: () => new Promise((ok, fail) => { ffmpeg.on("close", (code) => code ? fail(new Error(`ffmpeg ${code}`)) : ok()); ffmpeg.stdin.end(); }) };
};

mkdirSync(join(here, "videos"), { recursive: true });
mkdirSync(join(here, "stills"), { recursive: true });
const browser = await chromium.launch();
const page = await browser.newPage();
await page.setContent(`<canvas id="c" width="${WIDTH}" height="${HEIGHT}"></canvas>`);
await page.addScriptTag({ content: readFileSync(join(here, "scenes.js"), "utf8") });

for (const [id, [scene, hue]] of Object.entries(CATALOG)) {
  const video = encode(join(here, "videos", `${id}.mp4`));
  for (let frame = 0; frame < FPS * SECONDS; frame += BATCH) {
    const jpegs = await page.evaluate(({ scene, hue, frame, count, fps }) => {
      const canvas = document.getElementById("c"), ctx = canvas.getContext("2d"), out = [];
      for (let i = frame; i < frame + count; i++) {
        SCENES[scene](ctx, canvas.width, canvas.height, i / fps, hue);
        out.push(canvas.toDataURL("image/jpeg", 0.92).split(",")[1]);
      }
      return out;
    }, { scene, hue, frame, count: Math.min(BATCH, FPS * SECONDS - frame), fps: FPS });
    for (const jpeg of jpegs) await video.write(Buffer.from(jpeg, "base64"));
  }
  await video.end();
  const still = await page.evaluate(({ scene, hue, at }) => {
    const canvas = document.getElementById("c"); SCENES[scene](canvas.getContext("2d"), canvas.width, canvas.height, at, hue);
    return canvas.toDataURL("image/jpeg", 0.85).split(",")[1];
  }, { scene, hue, at: STILL_AT });
  writeFileSync(join(here, "stills", `${id}.jpg`), Buffer.from(still, "base64"));
  console.log(`${id}: ${scene}`);
}
await browser.close();
