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
  curse(c, W, H, t, h) {
    const g = c.createRadialGradient(W * .68, H * .45, 0, W * .68, H * .45, W * .7); g.addColorStop(0, `hsl(${h} 60% 22%)`); g.addColorStop(1, "#06030d"); c.fillStyle = g; c.fillRect(0, 0, W, H);
    c.globalCompositeOperation = "lighter";
    for (let k = 0; k < 7; k++) { const r = H * (.12 + k * .06) * (1 + .04 * Math.sin(t * 3 + k)), a = t * (k % 2 ? 1.4 : -1) + k;
      c.strokeStyle = `hsla(${h + k * 12} 100% ${60 + k * 3}% / ${.55 - k * .05})`; c.lineWidth = H * .012; c.beginPath(); c.arc(W * .68, H * .45, r, a, a + 2.4); c.stroke(); }
    const core = c.createRadialGradient(W * .68, H * .45, 0, W * .68, H * .45, H * .16); core.addColorStop(0, `hsla(${h + 40} 100% 85% / ${.8 + .2 * Math.sin(t * 8)})`); core.addColorStop(1, "transparent"); c.fillStyle = core; c.fillRect(0, 0, W, H);
    const r = seeded(9);
    for (let i = 0; i < 60; i++) { const a = r() * 7 + t * (.5 + r()), d = H * (.1 + ((r() + t * .3) % 1) * .5); c.fillStyle = `hsla(${h + 30} 100% 75% / ${r()})`; c.fillRect(W * .68 + Math.cos(a) * d, H * .45 + Math.sin(a) * d, 3, 3); }
    c.globalCompositeOperation = "source-over";
  },
  petals(c, W, H, t, h) {
    const g = c.createLinearGradient(0, 0, 0, H); g.addColorStop(0, `hsl(${h} 70% 88%)`); g.addColorStop(1, `hsl(${h + 20} 60% 72%)`); c.fillStyle = g; c.fillRect(0, 0, W, H);
    c.fillStyle = `hsl(${h - 120} 15% 92%)`; c.fillRect(W * .45, H * .48, W * .5, H * .52); c.fillStyle = `hsl(${h - 120} 20% 80%)`; c.fillRect(W * .62, H * .36, W * .12, H * .14);
    c.fillStyle = `hsl(${h - 140} 35% 60%)`; for (let y = 0; y < 3; y++) for (let x = 0; x < 6; x++) c.fillRect(W * (.48 + x * .08), H * (.56 + y * .13), W * .05, H * .07);
    c.fillStyle = "#fff"; c.beginPath(); c.ellipse(W * .3, H * .3, W * .1, H * .09, 0, 0, 7); c.fill(); c.beginPath(); c.moveTo(W * .3, H * .38); c.lineTo(W * .33, H * .45); c.lineTo(W * .34, H * .37); c.fill();
    c.fillStyle = `hsl(${h} 40% 35%)`; for (let i = 0; i < 3; i++) { const on = (Math.floor(t * 2) % 4) > i; if (on) { c.beginPath(); c.arc(W * (.26 + i * .04), H * .3, H * .014, 0, 7); c.fill(); } }
    const r = seeded(4);
    for (let i = 0; i < 70; i++) { const x = (r() * W + t * 40 * (.5 + r()) + Math.sin(t + i) * 30) % W, y = (r() * H + t * 30 * (.5 + r())) % H;
      c.save(); c.translate(x, y); c.rotate(t * 2 + i); c.fillStyle = `hsla(${h} 90% ${80 + r() * 10}% / .9)`; c.beginPath(); c.ellipse(0, 0, 7, 4, 0, 0, 7); c.fill(); c.restore(); }
  },
  leaves(c, W, H, t, h) {
    const g = c.createLinearGradient(0, 0, 0, H); g.addColorStop(0, `hsl(${h + 10} 90% 62%)`); g.addColorStop(.6, `hsl(${h - 10} 85% 50%)`); g.addColorStop(1, `hsl(${h - 20} 70% 25%)`); c.fillStyle = g; c.fillRect(0, 0, W, H);
    c.fillStyle = `hsla(${h + 25} 100% 85% / .9)`; c.beginPath(); c.arc(W * .7, H * .42, H * .14, 0, 7); c.fill();
    c.fillStyle = "#3b2414"; c.beginPath(); c.moveTo(0, H); for (let x = 0; x <= W; x += W / 10) c.lineTo(x, H * (.7 + .06 * Math.sin(x * .01))); c.lineTo(W, H); c.fill();
    c.fillStyle = "#24160c"; for (let i = 0; i < 6; i++) c.fillRect(W * (.08 + i * .17), H * .52, W * .012, H * .25);
    const run = (t * .25) % 1; c.strokeStyle = "#fff8"; c.lineWidth = 2; for (let k = 0; k < 4; k++) { c.beginPath(); c.moveTo(W * run - 80 - k * 18, H * (.62 + k * .02)); c.lineTo(W * run - 20 - k * 18, H * (.62 + k * .02)); c.stroke(); }
    c.fillStyle = "#1a1008"; c.fillRect(W * run - 14, H * .58, 14, 30);
    const r = seeded(13);
    for (let i = 0; i < 45; i++) { const a = t * (1 + r()) + i, d = H * (.08 + r() * .35), x = W * .5 + Math.cos(a) * d * 1.6, y = H * .45 + Math.sin(a) * d * .6;
      c.save(); c.translate(x, y); c.rotate(a * 2); c.fillStyle = i % 3 ? "#5fb83a" : "#8fd14f"; c.beginPath(); c.ellipse(0, 0, 9, 4, 0, 0, 7); c.fill(); c.restore(); }
  },
  walls(c, W, H, t, h) {
    const g = c.createLinearGradient(0, 0, 0, H); g.addColorStop(0, `hsl(${h} 25% 72%)`); g.addColorStop(1, `hsl(${h + 10} 20% 52%)`); c.fillStyle = g; c.fillRect(0, 0, W, H);
    const rise = H * (.14 + .05 * Math.sin(t * .8)), fx = W * .62, fy = H * .5 - rise;
    c.fillStyle = "#e7b89a"; c.beginPath(); c.arc(fx, fy, H * .2, 0, 7); c.fill();
    c.fillStyle = "#4a2b1c"; c.beginPath(); c.arc(fx, fy - H * .07, H * .205, Math.PI, 0); c.fill();
    const blink = Math.sin(t * 2.5) > .95 ? .2 : 1; c.fillStyle = "#1c1210"; c.beginPath(); c.ellipse(fx - H * .07, fy, H * .02, H * .025 * blink, 0, 0, 7); c.ellipse(fx + H * .07, fy, H * .02, H * .025 * blink, 0, 0, 7); c.fill();
    c.strokeStyle = "#7a3b2a"; c.lineWidth = H * .012; c.beginPath(); c.arc(fx, fy + H * .06, H * .06, .2, Math.PI - .2); c.stroke();
    const wave = Math.sin(t * 4) * .4; c.save(); c.translate(fx + H * .3, H * .5); c.rotate(-.4 + wave); c.fillStyle = "#e7b89a"; c.fillRect(-H * .04, -H * .22, H * .08, H * .22); c.beginPath(); c.arc(0, -H * .24, H * .06, 0, 7); c.fill(); c.restore();
    c.fillStyle = `hsl(${h + 15} 15% 38%)`; c.fillRect(0, H * .5, W, H * .5);
    c.strokeStyle = `hsl(${h + 15} 15% 28%)`; c.lineWidth = 2; for (let y = H * .5; y < H; y += H * .06) { c.beginPath(); c.moveTo(0, y); c.lineTo(W, y); c.stroke(); }
    for (let row = 0; row * H * .06 < H * .5; row++) for (let x = (row % 2) * W * .04; x < W; x += W * .08) { c.beginPath(); c.moveTo(x, H * .5 + row * H * .06); c.lineTo(x, H * .5 + (row + 1) * H * .06); c.stroke(); }
    c.strokeStyle = "#2228"; c.lineWidth = 2; for (let i = 0; i < 5; i++) { const bx = ((t * 40 + i * 230) % (W + 100)) - 50, by = H * (.12 + .05 * (i % 3)); c.beginPath(); c.moveTo(bx - 8, by - 4); c.lineTo(bx, by); c.lineTo(bx + 8, by - 4); c.stroke(); }
  },
};
globalThis.SCENES = SCENES;
