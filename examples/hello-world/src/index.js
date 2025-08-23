import { app, BrowserWindow } from 'nanoframe'

async function main() {
  await app.whenReady
  const win = await BrowserWindow.create({
    title: 'Nanoframe Hello',
    width: 800,
    height: 600,
    url: "https://example.com"
  })
  // demo: evaluate JS
  await win.eval('console.log("Hello from nanoframe")')
  await win.openDevTools()
  await win.center()
  await win.setAlwaysOnTop(false)
}

main().catch(err => {
  console.error(err)
  process.exit(1)
})
