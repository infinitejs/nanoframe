import { app, BrowserWindow } from 'nanoframe';
import { fileURLToPath } from "node:url";
import { dirname } from "node:path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function main() {
  const win = await BrowserWindow.create({
    title: 'Nanoframe Hello',
    width: 800,
    height: 600,
    url: "https://example.com",
  });

  await app.whenReady;

  await win.setIcon(`${__dirname}/assets/logo.png`);
  await win.eval('console.log("Hello from nanoframe")');
  await win.openDevTools();
  await win.center();
  await win.setAlwaysOnTop(false);
  await win.show();
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
