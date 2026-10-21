import { chromium } from 'playwright';

const browser = await chromium.launch();
const page = await browser.newPage();
page.setViewportSize({ width: 1440, height: 900 });

const pages = [
  { url: 'http://localhost:4321/', file: 'home-desktop.png' },
  { url: 'http://localhost:4321/library', file: 'library-desktop.png' },
];

for (const p of pages) {
  try {
    console.log(`📸 Fetching ${p.url}...`);
    await page.goto(p.url, { waitUntil: 'networkidle', timeout: 15000 });
    await page.screenshot({ path: `../../docs/screenshots/${p.file}` });
    console.log(`✅ ${p.file}`);
  } catch (e) {
    console.error(`❌ ${p.file}: ${e.message}`);
  }
}

await browser.close();
console.log('Done.');
