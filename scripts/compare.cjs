const { chromium } = require('playwright');
const { mkdirSync, readFileSync, writeFileSync } = require('node:fs');
const { basename, join, resolve } = require('node:path');
const { tmpdir } = require('node:os');
const { pathToFileURL } = require('node:url');

const TOLERANCE = 3;
const MARGIN = 4;

async function main() {
    const [artifact, reference, outDir = join(tmpdir(), 'barrierkit-compare')] = process.argv.slice(2);
    if (!artifact || !reference) {
        console.error('usage: node scripts/compare.cjs <artifact.html> <reference.png> [out-dir]');
        process.exit(2);
    }

    const referencePng = readFileSync(reference);
    const width = referencePng.readUInt32BE(16);
    const height = referencePng.readUInt32BE(20);
    const colorScheme = basename(reference).includes('-dark') ? 'dark' : 'light';

    const browser = await chromium.launch();
    const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: 1, colorScheme });
    await page.goto(pathToFileURL(resolve(artifact)).href);
    await page.evaluate(() => document.fonts.ready);
    const controls = await page.$$eval('[data-bk]', (elements) =>
        elements.map((element) => {
            const box = element.getBoundingClientRect();
            return { name: element.dataset.bk, x: box.x, y: box.y, width: box.width, height: box.height };
        }),
    );
    const artifactPng = await page.screenshot();

    const report = await page.evaluate(
        async ({ referenceUrl, artifactUrl, controls, tolerance, margin }) => {
            const pixels = async (url) => {
                const image = new Image();
                image.src = url;
                await image.decode();
                const canvas = new OffscreenCanvas(image.width, image.height);
                const context = canvas.getContext('2d');
                context.drawImage(image, 0, 0);
                return context.getImageData(0, 0, image.width, image.height);
            };
            const reference = await pixels(referenceUrl);
            const artifact = await pixels(artifactUrl);
            const { width, height } = reference;
            const at = (image, x, y) => {
                const i = (y * width + x) * 4;
                return [image.data[i], image.data[i + 1], image.data[i + 2], image.data[i + 3]];
            };
            const distance = (a, b) => Math.max(...a.map((channel, i) => Math.abs(channel - b[i])));

            const diff = new ImageData(width, height);
            let differing = 0;
            for (let y = 0; y < height; y++) {
                for (let x = 0; x < width; x++) {
                    const a = at(reference, x, y);
                    const different = distance(a, at(artifact, x, y)) > tolerance;
                    const grey = (a[0] + a[1] + a[2]) / 12 + 192;
                    const i = (y * width + x) * 4;
                    diff.data.set(different ? [255, 0, 0, 255] : [grey, grey, grey, 255], i);
                    if (different) differing++;
                }
            }

            const inside = (area) => ({
                x0: Math.max(0, Math.floor(area.x)),
                y0: Math.max(0, Math.floor(area.y)),
                x1: Math.min(width, Math.ceil(area.x + area.width)),
                y1: Math.min(height, Math.ceil(area.y + area.height)),
            });
            const measure = (image, area) => {
                const { x0, y0, x1, y1 } = inside(area);
                const background = at(image, x0, y0);
                const counts = new Map();
                let box = null;
                for (let y = y0; y < y1; y++) {
                    for (let x = x0; x < x1; x++) {
                        const pixel = at(image, x, y);
                        if (distance(pixel, background) <= tolerance) continue;
                        box = box
                            ? { x0: Math.min(box.x0, x), y0: Math.min(box.y0, y), x1: Math.max(box.x1, x), y1: Math.max(box.y1, y) }
                            : { x0: x, y0: y, x1: x, y1: y };
                        const key = pixel.join(',');
                        counts.set(key, (counts.get(key) ?? 0) + 1);
                    }
                }
                if (!box) return null;
                const fill = [...counts].sort((a, b) => b[1] - a[1])[0][0].split(',').map(Number);
                return { x: box.x0, y: box.y0, width: box.x1 - box.x0 + 1, height: box.y1 - box.y0 + 1, fill };
            };
            const compare = (name, area) => {
                const a = measure(reference, area);
                const b = measure(artifact, area);
                const { x0, y0, x1, y1 } = inside(area);
                let bad = 0;
                for (let y = y0; y < y1; y++)
                    for (let x = x0; x < x1; x++) if (distance(at(reference, x, y), at(artifact, x, y)) > tolerance) bad++;
                const total = Math.max(1, (x1 - x0) * (y1 - y0));
                return {
                    name,
                    differing: +((100 * bad) / total).toFixed(2),
                    dx: a && b ? b.x - a.x : null,
                    dy: a && b ? b.y - a.y : null,
                    dwidth: a && b ? b.width - a.width : null,
                    dheight: a && b ? b.height - a.height : null,
                    dfill: a && b ? distance(a.fill, b.fill) : null,
                    reference: a,
                    artifact: b,
                };
            };

            const canvas = new OffscreenCanvas(width, height);
            canvas.getContext('2d').putImageData(diff, 0, 0);
            const blob = await canvas.convertToBlob({ type: 'image/png' });
            const bytes = new Uint8Array(await blob.arrayBuffer());
            let binary = '';
            for (const byte of bytes) binary += String.fromCharCode(byte);

            return {
                width,
                height,
                differing: +((100 * differing) / (width * height)).toFixed(2),
                rows: [
                    compare('(whole)', { x: 0, y: 0, width, height }),
                    ...controls.map((c) =>
                        compare(c.name, { x: c.x - margin, y: c.y - margin, width: c.width + 2 * margin, height: c.height + 2 * margin }),
                    ),
                ],
                diffPng: btoa(binary),
            };
        },
        {
            referenceUrl: `data:image/png;base64,${referencePng.toString('base64')}`,
            artifactUrl: `data:image/png;base64,${artifactPng.toString('base64')}`,
            controls,
            tolerance: TOLERANCE,
            margin: MARGIN,
        },
    );
    await browser.close();

    mkdirSync(outDir, { recursive: true });
    writeFileSync(`${outDir}/artifact.png`, artifactPng);
    writeFileSync(`${outDir}/diff.png`, Buffer.from(report.diffPng, 'base64'));
    delete report.diffPng;
    writeFileSync(`${outDir}/report.json`, `${JSON.stringify(report, null, 2)}\n`);

    const cell = (value) => (value === null ? '–' : value);
    console.log(`${basename(reference)} (${report.width}×${report.height}, ${colorScheme}): ${report.differing}% of pixels differ`);
    console.log('| control | % differ | Δx | Δy | Δwidth | Δheight | Δfill |');
    console.log('|---|---|---|---|---|---|---|');
    for (const r of report.rows)
        console.log(`| ${r.name} | ${r.differing} | ${cell(r.dx)} | ${cell(r.dy)} | ${cell(r.dwidth)} | ${cell(r.dheight)} | ${cell(r.dfill)} |`);
    console.log(`\n${outDir}/artifact.png, ${outDir}/diff.png, ${outDir}/report.json`);
}

main().catch((error) => {
    console.error(error);
    process.exit(1);
});
