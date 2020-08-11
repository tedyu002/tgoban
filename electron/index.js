const { app, BrowserWindow } = require('electron')

function createWindow () {
	// Create the browser window.
	let win = new BrowserWindow({
		width: 900,
		height: 900,
		webPreferences: {
			nodeIntegration: true
		}
	})

	// and load the index.html of the app.
	win.loadFile('dist/index.html')
	win.webContents.openDevTools()
}

app.whenReady().then(createWindow)
