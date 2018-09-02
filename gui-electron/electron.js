"use strict";

const { app, Menu, BrowserWindow, dialog, ipcMain } = require("electron");
const fs = require("fs");

// hot-load for static files
require("electron-reload")(__dirname);

let mainWindow = null;

app.on("ready", () => {
  mainWindow = new BrowserWindow({
    width: 1280,
    height: 800
  });

  /*
  const startUrl = process.env.ELECTRON_START_URL || url.format({
    pathname: path.join(__dirname, '/../public/index.html'),
    protocol: 'file:',
    slashes: true
  });
  */
  mainWindow.loadURL("http://localhost:3001");

  mainWindow.webContents.openDevTools();

  mainWindow.on("closed", function() {
    mainWindow = null;
  });
});

app.on("window-all-closed", function() {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

app.on("activate", function() {
  if (mainWindow === null) {
    createWindow();
  }
});

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
