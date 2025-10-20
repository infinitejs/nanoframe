import { app, BrowserWindow } from 'nanoframe';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { writeFile } from 'node:fs/promises';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function main() {
  // Create hidden then show after we tweak bounds and sizes
  const win = await BrowserWindow.create({
    title: 'Nanoframe Hello',
    width: 1000,
    height: 700,
    x: 150,
    y: 100,
    show: false,
    minWidth: 800,
    minHeight: 500,
    url: 'https://example.com',
  });

  await app.whenReady;

  await win.setIcon(`${__dirname}/assets/logo.png`);
  await win.eval('console.log("Hello from nanoframe")');
  await win.setAlwaysOnTop(false);

  // Demonstrate bounds API
  await win.setBounds({ width: 1024, height: 640 });
  const bounds = await win.getBounds();
  console.log('Bounds after resize:', bounds);

  // Show it now
  await win.show();

  // Enable a basic custom context menu hook
  await win.enableContextMenu();
  app.on('webviewIpc', async ({ windowId, payload }) => {
    if (windowId !== win.id) return;
    if (payload?.type === 'context-menu') {
      const { x, y } = payload.detail || {};
      // Inject a minimal in-page menu (pure HTML/CSS for demo). Click outside to dismiss.
      await win.eval(`(function(){
        const id='__nf_demo_menu';
        const old=document.getElementById(id); if (old) old.remove();
        const m=document.createElement('div');
        m.id=id; m.style.cssText='position:fixed; z-index:99999; background:#222; color:#eee; box-shadow:0 6px 24px rgba(0,0,0,.3); border:1px solid #444; font:14px system-ui;';
        m.style.left='${x}px'; m.style.top='${y}px';
        const item=(txt,cb)=>{ const it=document.createElement('div'); it.textContent=txt; it.style.cssText='padding:8px 14px; cursor:pointer; white-space:nowrap;'; it.onmouseenter=()=>it.style.background='#333'; it.onmouseleave=()=>it.style.background='transparent'; it.onclick=()=>{ cb(); cleanup(); }; return it; };
        const cleanup=()=>{ m.remove(); document.removeEventListener('click', cleanup, true); };
        m.appendChild(item('Say Hi', ()=>alert('Hello!')));
        m.appendChild(item('Open DevTools', ()=>window.open('about:blank')));
        document.body.appendChild(m);
        setTimeout(()=>document.addEventListener('click', cleanup, true), 0);
      })();`);
    }
  });

  // Showcase maximize/unmaximize + restore
  await win.maximize();
  console.log('isMaximized:', await win.isMaximized());
  await win.unmaximize();
  await win.restore();

  // Request user attention (Windows taskbar flash)
  await win.requestUserAttention(false);

  // Take a screenshot (base64 PNG) and save to temp folder
  const { base64Png } = await win.screenshot();
  const { path: tempDir } = await app.getPath('temp');
  const out = join(tempDir, 'nanoframe-screenshot.png');
  await writeFile(out, Buffer.from(base64Png, 'base64'));
  console.log('Saved screenshot to', out);
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
