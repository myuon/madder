import { app, BrowserWindow, Menu, dialog, ipcMain } from 'electron'
import fs from 'fs'

/**
 * Set `__static` path to static files in production
 * https://simulatedgreg.gitbooks.io/electron-vue/content/en/using-static-assets.html
 */
if (process.env.NODE_ENV !== 'development') {
  global.__static = require('path').join(__dirname, '/static').replace(/\\/g, '\\\\')
}

let mainWindow
const winURL = process.env.NODE_ENV === 'development'
  ? `http://localhost:9080`
  : `file://${__dirname}/index.html`

function createWindow () {
  /**
   * Initial window options
   */
  mainWindow = new BrowserWindow({
    height: 563,
    useContentSize: true,
    width: 1000
  })

  mainWindow.loadURL(winURL)

  mainWindow.on('closed', () => {
    mainWindow = null
  })
}

app.on('ready', createWindow)

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})

app.on('activate', () => {
  if (mainWindow === null) {
    createWindow()
  }
})


const templateMenu = [
  {
    label: "File",
    submenu: [
      {
        label: "Open",
        click(item, focusedWindow) {
          dialog.showOpenDialog(
            mainWindow,
            {
              title: "open yaml file"
            },
            paths => {
              if (paths != null && paths.length > 0) {
                const path = paths[0];
                fs.readFile(path, "utf8", (err, data) => {
                  mainWindow.webContents.send("open-yaml", data);
                });
              }
            }
          );
        }
      },
      {
        label: "Save",
        click(item, focusedWindow) {
          dialog.showSaveDialog(
            mainWindow,
            {
              title: "save yaml file"
            },
            path => {
              console.log(path);
              if (path != null) {
                mainWindow.webContents.send("request-save-yaml", "");
                ipcMain.on("response-save-yaml", (event, arg) => {
                  fs.writeFile(path, JSON.parse(arg));
                });
              }
            }
          );
        }
      }
    ]
  },
  {
    label: "Edit",
    submenu: [
      {
        role: "undo"
      },
      {
        role: "redo"
      }
    ]
  },
  {
    label: "View",
    submenu: [
      {
        label: "Reload",
        accelerator: "CmdOrCtrl+R",
        click(item, focusedWindow) {
          if (focusedWindow) focusedWindow.reload();
        }
      },
      {
        type: "separator"
      },
      {
        role: "resetzoom"
      },
      {
        role: "zoomin"
      },
      {
        role: "zoomout"
      },
      {
        type: "separator"
      },
      {
        role: "togglefullscreen"
      }
    ]
  }
];

const menu = Menu.buildFromTemplate(templateMenu);
Menu.setApplicationMenu(menu);

/**
 * Auto Updater
 *
 * Uncomment the following code below and install `electron-updater` to
 * support auto updating. Code Signing with a valid certificate is required.
 * https://simulatedgreg.gitbooks.io/electron-vue/content/en/using-electron-builder.html#auto-updating
 */

/*
import { autoUpdater } from 'electron-updater'

autoUpdater.on('update-downloaded', () => {
  autoUpdater.quitAndInstall()
})

app.on('ready', () => {
  if (process.env.NODE_ENV === 'production') autoUpdater.checkForUpdates()
})
 */
