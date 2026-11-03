// Procedural scenes of the Marea mockup; generate.mjs renders them into the test videos.
const seeded = s => () => (s = (s * 16807) % 2147483647) / 2147483647;
const SCENES = {
  heat(c, W, H, t, h) {
    const g = c.createLinearGradient(0, 0, 0, H); g.addColorStop(0, `hsl(${h} 70% 62%)`); g.addColorStop(.55, `hsl(${h + 15} 80% 55%)`); g.addColorStop(.56, `hsl(${h + 5} 55% 45%)`); g.addColorStop(1, `hsl(${h - 5} 50% 22%)`);
    c.fillStyle = g; c.fillRect(0, 0, W, H);
    c.fillStyle = `hsla(${h + 30} 100% 85% / .9)`; c.beginPath(); c.arc(W * .72, H * .32, H * .12, 0, 7); c.fill();
    for (let y = H * .56; y < H; y += 6) { c.fillStyle = `hsla(${h} 60% 70% / .08)`; c.fillRect(Math.sin(y * .05 + t * 3) * 14, y, W, 2); }
    c.fillStyle = "#1a1208"; const x = (t * 30) % (W + 200) - 100; c.fillRect(x, H * .6, 70, 22); c.fillRect(x + 12, H * .6 - 14, 34, 14);
  },
  flicker(c, W, H, t, h) {
    c.fillStyle = "#0a0608"; c.fillRect(0, 0, W, H);
    for (let i = 0; i < 14; i++) {
      const x = W * (i + .5) / 14, y = H * .28 + Math.sin(i * .9) * H * .05, on = Math.sin(t * 9 + i * 3.1) + Math.sin(t * 23 + i) > .3;
      const col = [0, 50, 120, 200, 300][i % 5], gl = c.createRadialGradient(x, y, 0, x, y, H * .22);
      gl.addColorStop(0, `hsla(${col} 100% 60% / ${on ? .7 : .08})`); gl.addColorStop(1, "transparent"); c.fillStyle = gl; c.fillRect(0, 0, W, H);
      c.fillStyle = on ? `hsl(${col} 100% 75%)` : "#333"; c.beginPath(); c.arc(x, y, Math.max(2, H * .012), 0, 7); c.fill();
    }
    c.strokeStyle = "#222"; c.beginPath(); for (let i = 0; i <= 14; i++) { const x = W * i / 14, y = H * .27 + Math.sin(i * .9) * H * .05; i ? c.lineTo(x, y) : c.moveTo(x, y); } c.stroke();
    c.fillStyle = `hsla(${h} 90% 50% / ${.06 + .05 * Math.sin(t * 2)})`; c.fillRect(0, 0, W, H);
  },
  rain(c, W, H, t, h) {
    const g = c.createLinearGradient(0, 0, W, H); g.addColorStop(0, `hsl(${h} 70% 22%)`); g.addColorStop(1, "#100"); c.fillStyle = g; c.fillRect(0, 0, W, H);
    const r = seeded(7);
    for (let i = 0; i < 70; i++) { const x = r() * W, sp = .4 + r(), y = ((r() * H + t * 60 * sp) % (H + 40)) - 20, a = t * 2 * sp + i;
      c.save(); c.translate(x + Math.sin(a) * 10, y); c.rotate(Math.sin(a) * .8); c.fillStyle = i % 3 ? "#cfe3c0" : "#e9e2c8"; c.fillRect(-W * .02, -W * .01, W * .04, W * .02); c.restore(); }
  },
  snow(c, W, H, t, h) {
    const g = c.createLinearGradient(0, 0, 0, H); g.addColorStop(0, "#0b1424"); g.addColorStop(1, `hsl(${h} 40% 18%)`); c.fillStyle = g; c.fillRect(0, 0, W, H);
    c.fillStyle = "#05080d"; c.beginPath(); c.moveTo(0, H); for (let x = 0; x <= W; x += W / 8) c.lineTo(x, H * (.62 + .12 * Math.sin(x * .02))); c.lineTo(W, H); c.fill();
    const r = seeded(3);
    for (let i = 0; i < 90; i++) { const x = (r() * W + Math.sin(t + i) * 20) % W, y = (r() * H + t * 25 * (.5 + r())) % H; c.fillStyle = "#ffffffcc"; c.fillRect(x, y, 2, 2); }
    for (let i = 0; i < 25; i++) { const x = r() * W, y = H - ((r() * H + t * 40) % H); c.fillStyle = `hsla(20 100% 60% / ${r()})`; c.fillRect(x + Math.sin(t * 3 + i) * 6, y, 3, 3); }
  },
  steam(c, W, H, t, h) {
    const g = c.createLinearGradient(0, 0, 0, H); g.addColorStop(0, `hsl(${h} 35% 18%)`); g.addColorStop(1, "#140c06"); c.fillStyle = g; c.fillRect(0, 0, W, H);
    c.fillStyle = "#2b2b2e"; c.fillRect(0, H * .72, W, H * .28); c.fillStyle = "#555"; c.fillRect(W * .35, H * .6, W * .3, H * .14);
    for (let i = 0; i < 18; i++) { const p = ((t * .25 + i / 18) % 1), x = W * .5 + Math.sin(i * 2 + t) * W * .08, y = H * .6 - p * H * .6, r = H * (.05 + p * .2);
      const gl = c.createRadialGradient(x, y, 0, x, y, r); gl.addColorStop(0, `rgba(255,255,255,${.25 * (1 - p)})`); gl.addColorStop(1, "transparent"); c.fillStyle = gl; c.fillRect(0, 0, W, H); }
    const fl = .6 + .4 * Math.sin(t * 13); c.fillStyle = `hsla(30 100% 55% / ${.3 * fl})`; c.fillRect(W * .37, H * .74, W * .26, 4);
  },
  grid(c, W, H, t, h) {
    c.fillStyle = `hsl(${h} 30% 90%)`; c.fillRect(0, 0, W, H);
    c.strokeStyle = `hsla(${h} 50% 35% / .5)`; c.lineWidth = 1;
    const vx = W / 2, vy = H * .45;
    for (let i = -10; i <= 10; i++) { c.beginPath(); c.moveTo(vx, vy); c.lineTo(vx + i * W * .15, H); c.stroke(); c.beginPath(); c.moveTo(vx, vy); c.lineTo(vx + i * W * .15, 0); c.stroke(); }
    for (let k = 0; k < 12; k++) { const p = ((k / 12 + t * .08) % 1), y = vy + (H - vy) * p * p, y2 = vy - vy * p * p; c.beginPath(); c.moveTo(0, y); c.lineTo(W, y); c.moveTo(0, y2); c.lineTo(W, y2); c.stroke(); }
    c.fillStyle = `hsl(${h} 60% 30%)`; const s = H * .05 * (1 + .1 * Math.sin(t * 2)); c.fillRect(vx - s / 2, vy - s, s, s * 2);
  },
  stadium(c, W, H, t, h) {
    c.fillStyle = "#07101a"; c.fillRect(0, 0, W, H);
    for (let i = 0; i < 10; i++) { c.fillStyle = i % 2 ? `hsl(${h} 55% 28%)` : `hsl(${h} 55% 32%)`; c.fillRect(0, H * .62 + i * H * .04, W, H * .04); }
    c.globalCompositeOperation = "lighter";
    for (let i = 0; i < 3; i++) { const a = Math.sin(t * .6 + i * 2) * .5, x = W * (.2 + i * .3);
      c.save(); c.translate(x, 0); c.rotate(a); const g = c.createLinearGradient(0, 0, 0, H); g.addColorStop(0, "rgba(255,255,220,.45)"); g.addColorStop(1, "rgba(255,255,220,0)"); c.fillStyle = g;
      c.beginPath(); c.moveTo(-6, 0); c.lineTo(6, 0); c.lineTo(W * .12, H); c.lineTo(-W * .12, H); c.fill(); c.restore(); }
    c.globalCompositeOperation = "source-over";
    c.fillStyle = "#fff"; const bx = W * .5 + Math.sin(t * 1.3) * W * .3; c.beginPath(); c.arc(bx, H * .75 - Math.abs(Math.sin(t * 3)) * H * .15, H * .02, 0, 7); c.fill();
  },
  glitch(c, W, H, t, h) {
    c.fillStyle = "#050505"; c.fillRect(0, 0, W, H);
    const step = Math.floor(t * 6), r = seeded(step + 11);
    c.fillStyle = "#e8e8e8"; c.font = `300 ${H * .18}px Geist, sans-serif`; c.textAlign = "center"; c.textBaseline = "middle";
    const jx = (r() - .5) * 10; c.globalCompositeOperation = "lighter";
    c.fillStyle = "#ff004c"; c.fillText("SEÑAL", W / 2 + jx - 3, H / 2); c.fillStyle = "#00e5ff"; c.fillText("SEÑAL", W / 2 + jx + 3, H / 2); c.fillStyle = "#ffffff"; c.fillText("SEÑAL", W / 2 + jx, H / 2);
    c.globalCompositeOperation = "source-over";
    for (let i = 0; i < 8; i++) { const y = r() * H, hh = r() * H * .05; c.drawImage(c.canvas, 0, y, W, hh, (r() - .5) * 60, y, W, hh); }
    c.strokeStyle = "#ffffff30"; c.beginPath(); c.moveTo(W * .3, 0); c.lineTo(W * .45, H * .4); c.lineTo(W * .38, H); c.moveTo(W * .45, H * .4); c.lineTo(W * .8, H * .2); c.stroke();
  },
  city(c, W, H, t, h) {
    const g = c.createLinearGradient(0, 0, 0, H); g.addColorStop(0, "#06070d"); g.addColorStop(1, `hsl(${h} 40% 16%)`); c.fillStyle = g; c.fillRect(0, 0, W, H);
    const r = seeded(5); let x = 0;
    while (x < W) { const bw = W * (.06 + r() * .08), bh = H * (.3 + r() * .55), off = (t * 6) % W; c.fillStyle = "#0b0d14"; c.fillRect(x, H - bh, bw, bh);
      for (let wy = H - bh + 8; wy < H - 6; wy += 9) for (let wx = x + 4; wx < x + bw - 4; wx += 7) { const on = Math.sin(wx * 13 + wy * 7 + Math.floor(t * .7 + wx)) > .2; if (on) { c.fillStyle = `hsla(${40 + (wx % 30)} 90% 70% / .8)`; c.fillRect(wx, wy, 3, 4); } }
      x += bw + 2; }
  },
  pixel(c, W, H, t, h) {
    const p = Math.max(4, Math.floor(H / 40)); c.imageSmoothingEnabled = false;
    c.fillStyle = `hsl(${h} 80% 62%)`; c.fillRect(0, 0, W, H);
    for (let i = 0; i < 5; i++) { const x = ((i * W / 4 - t * 20) % (W + 100) + W + 100) % (W + 100) - 60, y = H * (.1 + .08 * (i % 3)); c.fillStyle = "#fff"; c.fillRect(x, y, p * 8, p * 2); c.fillRect(x + p * 2, y - p, p * 4, p); }
    const off = (t * 60) % (p * 4);
    for (let x = -off; x < W; x += p * 4) { c.fillStyle = "#8b4a1b"; c.fillRect(x, H * .8, p * 4, H * .2); c.fillStyle = "#3dbb3d"; c.fillRect(x, H * .8, p * 4, p); c.fillStyle = "#6b3412"; c.fillRect(x + p, H * .8 + p * 2, p, p); }
    const jump = Math.abs(Math.sin(t * 2.2)) * H * .18; c.fillStyle = "#e33"; c.fillRect(W * .3, H * .8 - p * 6 - jump, p * 4, p * 6); c.fillStyle = "#ffd39b"; c.fillRect(W * .3 + p, H * .8 - p * 5 - jump, p * 2, p * 2);
    c.fillStyle = "#ffd700"; c.fillRect(((W - t * 60 % W) + W) % W, H * .5, p * 2, p * 2);
  },
  fog(c, W, H, t, h) {
    c.fillStyle = `hsl(${h} 35% 9%)`; c.fillRect(0, 0, W, H);
    c.fillStyle = "#f4f0d8"; c.beginPath(); c.arc(W * .7, H * .28, H * .11, 0, 7); c.fill();
    c.fillStyle = "#03030a"; for (let i = 0; i < 12; i++) { const x = W * i / 11, th = H * (.3 + (i % 3) * .1); c.beginPath(); c.moveTo(x - W * .05, H); c.lineTo(x, H - th - H * .3); c.lineTo(x + W * .05, H); c.fill(); }
    for (let i = 0; i < 6; i++) { const x = ((t * 15 * (i + 1) + i * 200) % (W * 1.6)) - W * .3, y = H * (.65 + i * .05), g = c.createRadialGradient(x, y, 0, x, y, W * .3); g.addColorStop(0, "rgba(200,200,230,.18)"); g.addColorStop(1, "transparent"); c.fillStyle = g; c.fillRect(0, 0, W, H); }
    if (Math.sin(t * 1.3) > .97) { c.fillStyle = "rgba(255,255,255,.25)"; c.fillRect(0, 0, W, H); }
  },
  waves(c, W, H, t, h) {
    const g = c.createLinearGradient(0, 0, 0, H); g.addColorStop(0, `hsl(${h + 10} 60% 70%)`); g.addColorStop(.45, `hsl(${h} 60% 55%)`); g.addColorStop(1, `hsl(${h} 70% 18%)`); c.fillStyle = g; c.fillRect(0, 0, W, H);
    for (let k = 0; k < 6; k++) { c.fillStyle = `hsla(${h} 70% ${40 - k * 4}% / .55)`; c.beginPath(); c.moveTo(0, H);
      for (let x = 0; x <= W; x += 8) c.lineTo(x, H * (.45 + k * .09) + Math.sin(x * .012 + t * (1 + k * .3) + k) * H * .025); c.lineTo(W, H); c.fill(); }
    c.fillStyle = "#fff"; const bx = (t * 12) % (W + 80) - 40, by = H * .47 + Math.sin(bx * .012 + t) * H * .025; c.beginPath(); c.moveTo(bx, by); c.lineTo(bx + 14, by); c.lineTo(bx + 7, by - 24); c.fill();
  },
  neon(c, W, H, t, h) {
    c.fillStyle = "#07040d"; c.fillRect(0, 0, W, H);
    c.globalCompositeOperation = "lighter";
    for (let i = 0; i < 6; i++) { const y = H * (.15 + i * .14), x = ((t * 90 * (1 + i * .2) + i * 300) % (W * 1.5)) - W * .25, g = c.createLinearGradient(x - W * .2, 0, x, 0);
      g.addColorStop(0, "transparent"); g.addColorStop(1, `hsla(${h + i * 25} 100% 60% / .8)`); c.fillStyle = g; c.fillRect(x - W * .2, y, W * .2, 3); }
    for (let i = 0; i < 5; i++) { const x = W * (.1 + i * .2), gl = c.createRadialGradient(x, H * .85, 0, x, H * .85, H * .3); gl.addColorStop(0, `hsla(${h + i * 40} 100% 55% / ${.3 + .2 * Math.sin(t * 2 + i)})`); gl.addColorStop(1, "transparent"); c.fillStyle = gl; c.fillRect(0, 0, W, H); }
    c.globalCompositeOperation = "source-over";
  },
};
globalThis.SCENES = SCENES;
