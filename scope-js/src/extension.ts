import * as vscode from "vscode";
import { commands, ExtensionContext, Uri, Webview, WebviewView, WebviewViewProvider, window, workspace } from "vscode";

import * as helpers from "./helpers.js";

export async function activate(context: ExtensionContext) {
  // Check if a terminal with the name "scope" already exists, otherwise create one
  const terminalExists = window.terminals.find((terminal) => terminal.name === "scope");
  const terminal = terminalExists ? terminalExists : window.createTerminal("scope");

  // Create our webview provider
  const provider = new ToolkitViewProvider(context.extensionUri, terminal);
  // await provider.instantiateWasm(context);

  context.subscriptions.push(
    window.registerWebviewViewProvider(ToolkitViewProvider.viewType, provider, {
      webviewOptions: { retainContextWhenHidden: true },
    })
  );
}

export class ToolkitViewProvider implements WebviewViewProvider {
  public static readonly viewType = "scope.toolkitView";

  private _view?: WebviewView;

  constructor(
    private readonly _extensionUri: Uri, // private readonly _wasmInstance: WebAssembly.Instance,
    private _terminal: vscode.Terminal
  ) {
    if (!vscode.workspace.workspaceFolders) {
      throw Error(
        "There is no working directory defined. This extension requires VSCode is run at the root of a foundry project."
      );
    }
  }

  public resolveWebviewView(
    webviewView: WebviewView,
    context: vscode.WebviewViewResolveContext,
    _token: vscode.CancellationToken
  ) {
    this._view = webviewView;

    webviewView.webview.options = {
      // Allow scripts in the webview
      enableScripts: true,
      localResourceRoots: [this._extensionUri],
    };

    // Load up our wasm binary in the webview html
    webviewView.webview.html = this._getHtmlForWebview(webviewView.webview);

    // Handle opening and closing of documents; rust side will then query for open files
    vscode.workspace.onDidOpenTextDocument((e) => {
      this.sendOpenOrClosedNotifToRust();
    });

    vscode.workspace.onDidCloseTextDocument((e) => {
      this.sendOpenOrClosedNotifToRust();
    });

    // Handle messages in our webview (received from either rust or VSCode)
    // Essentially a function selector
    webviewView.webview.onDidReceiveMessage(async (message) => {
      const handleMessage = async (message: any) => {
        console.log(message);
        switch (message.command) {
          case "get_open_files": {
            // get filepaths of compiled .json files for whichever solidity files are open in the editor
            let compiledContracts = await helpers.getCompiledForOpenFiles();
            await this.sendOpenFilesToRust(compiledContracts);
            break;
          }

          case "get_file_contents": {
            const contents: string = (await helpers.loadFile(Uri.parse(message.data?.filePath))).toString();
            await this.sendFileContentsToRust(contents); // TODO: do we need anything other than compiled solidity?
            break;
          }

          case "get_compiled_solidity": {
            const contents: string = (await helpers.loadFile(Uri.parse(message.data.filePath))).toString();
            await this.sendCompiledSolidityToRust(contents, message.data.filePath);
            break;
          }

          // TODO: handle this better
          case "forge_build": {
            // hacky way for wasm side to know when a vscode terminal command completes
            await helpers.callTerminalHandleExit("forge build");
            await this.sendCompletedCompileNotifToRust();
            break;
          }

          // TODO: handle this better
          case "execute_shell_command": {
            if (this._terminal.exitStatus) {
              this._terminal.dispose();
              this._terminal = window.createTerminal("scope");
            }
            this._terminal.show();
            this._terminal.sendText(message.data.command);
            break;
          }

          case "error_popup": {
            // these error formats kinda suck
            // vscode.window.showWarningMessage(message.data.errorText);
            vscode.window.withProgress(
              { location: vscode.ProgressLocation.Notification, cancellable: true },
              (progress) => {
                progress.report({ message: message.data.text });
                return new Promise<void>((resolve) => {
                  setTimeout(() => {
                    resolve();
                  }, 4000);
                });
              }
            );
            break;
          }

          // Currently same as error popup, but could be different in the future
          case "ok_popup": {
            vscode.window.withProgress(
              { location: vscode.ProgressLocation.Notification, cancellable: true },
              (progress) => {
                progress.report({ message: message.data.text });
                return new Promise<void>((resolve) => {
                  setTimeout(() => {
                    resolve();
                  }, 4000);
                });
              }
            );
            break;
          }

          // TODO delete?
          case "webviewBlurred": {
            this.sendLostFocusToRust();
            break;
          }
          case "webviewFocused": {
            this.sendRegainedFocusToRust();
            break;
          }
        }
      };

      await handleMessage(message);
    });
  }

  private async sendOpenFilesToRust(solFiles: string[]) {
    // console.log("in sendopenfilestorust");
    this._view?.webview.postMessage({ command: "post_open_file_paths", content: solFiles });
  }

  private async sendFileContentsToRust(fileContents: string) {
    // console.log("in sendfilecontentstorust");
    this._view?.webview.postMessage({
      command: "post_file_contents",
      content: { fileContents: fileContents },
    });
  }

  private async sendCompiledSolidityToRust(compiledJson: string, filePath: string) {
    console.log("in sendcompiledsoliditytorust");
    this._view?.webview.postMessage({
      command: "post_compiled_solidity",
      content: { compiledJson: compiledJson, filePath: filePath },
    });
  }
  private async sendCompletedCompileNotifToRust() {
    console.log("in sendCompletedCompileNotif");
    this._view?.webview.postMessage({
      command: "completed_forge_build",
      content: {},
    });
  }
  private async sendLostFocusToRust() {
    console.log("in sendLostFocusToRust");
    this._view?.webview.postMessage({
      command: "lost_focus",
      content: {},
    });
  }

  private async sendRegainedFocusToRust() {
    console.log("in sendRegainedFocusToRust");
    this._view?.webview.postMessage({
      command: "gained_focus",
      content: {},
    });
  }

  private async sendOpenOrClosedNotifToRust() {
    console.log("in sendFileOpenedNotifToRust");
    this._view?.webview.postMessage({
      command: "file_opened_or_closed",
      content: {},
    });
  }

  private _getHtmlForWebview(webview: Webview) {
    const isDarkTheme =
      vscode.window.activeColorTheme.kind === vscode.ColorThemeKind.Dark ||
      vscode.window.activeColorTheme.kind === vscode.ColorThemeKind.HighContrast;

    const ethToolkitWasm = webview.asWebviewUri(Uri.joinPath(this._extensionUri, "wbg_out", "eth_toolkit_bg.wasm"));
    const ethToolkitJs = webview.asWebviewUri(Uri.joinPath(this._extensionUri, "wbg_out", "eth_toolkit.js"));
    const bridge = webview.asWebviewUri(
      Uri.joinPath(this._extensionUri, "wbg_out", "snippets", "eth_toolkit-eb19683a8f12ca3e", "src", "bridge.js")
    );

    // Use a nonce to only allow a specific script to be run.
    // const nonce = getNonce();
    //<meta http-equiv="Content-Security-Policy" content="default-src *; style-src 'unsafe-inline' *; script-src 'unsafe-inline' 'unsafe-eval' *; connect-src *;">
    // connect-src 'self' vscode-resource: http://127.0.0.1:8545 ">
    return `<!DOCTYPE html><html><head><meta http-equiv="Content-Type" content="text/html; charset=utf-8">

		<meta http-equiv="Content-Security-Policy" 
      content="default-src 'self';
               script-src 'self' 'unsafe-inline' 'unsafe-eval' vscode-resource: file: ;
               style-src 'self' 'unsafe-inline';
               connect-src 'self' vscode-resource: file http://127.0.0.1:8545 ">



		<!-- Disable zooming: -->
		<meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
		<meta name="theme-color" media="(prefers-color-scheme: light)" content="#404040">
		<meta name="theme-color" media="(prefers-color-scheme: dark)" content="#404040">


    <script> var isDarkTheme = ${isDarkTheme}; </script>

		<style>
			html {
				/* Remove touch delay: */
				touch-action: manipulation;
			}
	
	
			/* Allow canvas to fill entire web page: */
			html,
			body {
				overflow: hidden;
				margin: 0 !important;
				padding: 0 !important;
				height: 100%;
				width: 100%;
			}
	
			/* Position canvas in center-top: */
			canvas {
				margin-right: auto;
				margin-left: auto;
				display: block;
				position: absolute;
				top: 0%;
				left: 50%;
				transform: translate(-50%, 0%);
			}
	
			.centered {
				margin-right: auto;
				margin-left: auto;
				display: block;
				position: absolute;
				top: 50%;
				left: 50%;
				transform: translate(-50%, -50%);
				color: #404040;
				font-size: 24px;
				font-family: Courier, Ubuntu-Light, Helvetica, sans-serif;
				text-align: center;
			}
	
		</style>
    <div id="dummyDiv" tabindex="0" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; z-index: -1;"></div>
		
			<!-- config for our rust wasm binary. go to https://trunkrs.dev/assets/#rust for more customization -->
			<script type="module">import init from "${ethToolkitJs}";init("${ethToolkitWasm}");</script>
 
			
			<canvas id="the_canvas_id"></canvas>
			
			<link rel="preload" href="${ethToolkitWasm}" as="fetch" type="application/wasm" crossorigin="">
			<link rel="preload href="${ethToolkitJs}"></head>
			
			<script type="module" src="${bridge}"></script>


			</html><!-- Powered by egui: https://github.com/emilk/egui/ -->`;
  }
}

function getNonce() {
  let text = "";
  const possible = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  for (let i = 0; i < 32; i++) {
    text += possible.charAt(Math.floor(Math.random() * possible.length));
  }
  return text;
}
