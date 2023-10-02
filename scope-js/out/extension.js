"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.ToolkitViewProvider = exports.activate = void 0;
const vscode = require("vscode");
const vscode_1 = require("vscode");
const helpers = require("./helpers.js");
// import * as solc from "solc";
// import NativeSolcPlugin from "./remix-vscode-compiler/native_solidity_plugin.js";
// import { ISources, CompilerInput, CompilerInputOptions } from "./remix-vscode-compiler/types";
// import { exec } from "child_process";
function activate(context) {
    return __awaiter(this, void 0, void 0, function* () {
        // Check if a terminal with the name "nkit" already exists, otherwise create one
        const terminalExists = vscode_1.window.terminals.find((terminal) => terminal.name === "scope");
        const terminal = terminalExists ? terminalExists : vscode_1.window.createTerminal("scope");
        const provider = new ToolkitViewProvider(context.extensionUri, terminal);
        // await provider.instantiateWasm(context);
        helpers.getTheme();
        context.subscriptions.push(vscode_1.window.registerWebviewViewProvider(ToolkitViewProvider.viewType, provider, {
            webviewOptions: { retainContextWhenHidden: true },
        }));
    });
}
exports.activate = activate;
class ToolkitViewProvider {
    constructor(_extensionUri, // private readonly _wasmInstance: WebAssembly.Instance,
    _terminal) {
        this._extensionUri = _extensionUri;
        this._terminal = _terminal;
        if (vscode.workspace.workspaceFolders) {
            this.cwd = vscode.workspace.workspaceFolders[0].uri;
        }
        else {
            throw Error("There is no working directory defined. This extension requires VSCode is run at the root of a foundry project.");
        }
    }
    resolveWebviewView(webviewView, context, _token) {
        this._view = webviewView;
        webviewView.webview.options = {
            // Allow scripts in the webview
            enableScripts: true,
            // enableCommandUris: true, can i use this to use a shell in wasm??
            localResourceRoots: [this._extensionUri],
        };
        webviewView.onDidChangeVisibility((e) => {
            console.log(e);
        });
        // Handle opening and closing of documents
        vscode.workspace.onDidOpenTextDocument((e) => {
            this.sendOpenOrClosedNotifToRust();
        });
        vscode.workspace.onDidCloseTextDocument((e) => {
            this.sendOpenOrClosedNotifToRust();
        });
        webviewView.webview.html = this._getHtmlForWebview(webviewView.webview);
        // Handle messages received from either rust or VSCode
        webviewView.webview.onDidReceiveMessage((message) => __awaiter(this, void 0, void 0, function* () {
            // console.log("in ondidreceivemessage");
            // function selector for rust -> js calls
            const handleMessage = (message) => __awaiter(this, void 0, void 0, function* () {
                var _a;
                console.log(message);
                switch (message.command) {
                    case "called_from_rust": {
                        console.log("called_from_rust");
                        break;
                    }
                    case "get_open_files": {
                        // get solidity filenames
                        console.log("in vscode handling get_open_files");
                        // let openFiles = await helpers.getOpenSolidityFiles();
                        let openFiles = yield helpers.getOpenCompiledFiles();
                        let compiledContracts = [""];
                        if (JSON.stringify(openFiles) !== JSON.stringify(compiledContracts)) {
                            const outPath = vscode_1.Uri.joinPath(this.cwd, "out");
                            compiledContracts = yield helpers.getOpenCompiledContracts(openFiles, outPath);
                        }
                        yield this.sendOpenFilesToRust(compiledContracts);
                        console.log("sent openfiles");
                        break;
                    }
                    case "get_file_contents": {
                        const contents = (yield helpers.loadFile(vscode_1.Uri.parse((_a = message.data) === null || _a === void 0 ? void 0 : _a.filePath))).toString();
                        // const compiledJson: string = await helpers.compileSoliditySource(contents);
                        console.log(contents);
                        yield this.sendFileContentsToRust(contents); // TODO: do we need anything other than compiled solidity?
                        break;
                    }
                    case "get_compiled_solidity": {
                        // handle solidity file specifically
                        // const compiledPath = Uri.joinPath(this.cwd, "out", message.data?.filePath);
                        console.log(message.data.filePath);
                        const contents = (yield helpers.loadFile(vscode_1.Uri.parse(message.data.filePath))).toString();
                        yield this.sendCompiledSolidityToRust(contents, message.data.filePath);
                        // console.log("compiling");
                        // console.time("compile");
                        // const compiledJson: string = await helpers.compileSoliditySource(contents);
                        // console.timeEnd("compile");
                        break;
                    }
                    case "forge_build": {
                        // hacky way for wasm side to know when a vscode terminal command completes
                        yield helpers.callTerminalHandleExit("forge build");
                        yield this.sendCompletedCompileNotifToRust();
                        break;
                    }
                    case "execute_shell_command": {
                        if (this._terminal.exitStatus) {
                            this._terminal.dispose();
                            this._terminal = vscode_1.window.createTerminal("scope");
                        }
                        this._terminal.show();
                        this._terminal.sendText(message.data.command);
                        break;
                    }
                    case "error_popup": {
                        // these error formats kinda suck
                        // vscode.window.showWarningMessage(message.data.errorText);
                        vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, cancellable: true }, (progress) => {
                            progress.report({ message: message.data.text });
                            return new Promise((resolve) => {
                                setTimeout(() => {
                                    resolve();
                                }, 4000);
                            });
                        });
                        break;
                    }
                    // Currently same as error popup, but could be different in the future
                    case "ok_popup": {
                        vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, cancellable: true }, (progress) => {
                            progress.report({ message: message.data.text });
                            return new Promise((resolve) => {
                                setTimeout(() => {
                                    resolve();
                                }, 4000);
                            });
                        });
                        break;
                    }
                    case "webviewBlurred": {
                        this.sendLostFocusToRust();
                        const document = yield vscode.workspace.openTextDocument();
                        // const editor = vscode.window.activeTextEditor;
                        // if (editor) {
                        //   const position = editor.selection.active;
                        //   var newPosition = position.with(position.line, 0);
                        //   var newSelection = new vscode.Selection(newPosition, newPosition);
                        //   editor.selection = newSelection;
                        //   console.log("newselection", editor.selection);
                        // } else {
                        //   console.log("no editor");
                        // }
                        break;
                    }
                    case "webviewFocused": {
                        this.sendRegainedFocusToRust();
                        break;
                    }
                }
            });
            yield handleMessage(message);
        }));
    }
    callRust() {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            // console.log("in callrust");
            (_a = this._view) === null || _a === void 0 ? void 0 : _a.webview.postMessage({ command: "call_to_rust" });
        });
    }
    sendOpenFilesToRust(solFiles) {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            // console.log("in sendopenfilestorust");
            (_a = this._view) === null || _a === void 0 ? void 0 : _a.webview.postMessage({ command: "post_open_file_paths", content: solFiles });
        });
    }
    sendFileContentsToRust(fileContents) {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            // console.log("in sendfilecontentstorust");
            (_a = this._view) === null || _a === void 0 ? void 0 : _a.webview.postMessage({
                command: "post_file_contents",
                content: { fileContents: fileContents },
            });
        });
    }
    sendCompiledSolidityToRust(compiledJson, filePath) {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            console.log("in sendcompiledsoliditytorust");
            (_a = this._view) === null || _a === void 0 ? void 0 : _a.webview.postMessage({
                command: "post_compiled_solidity",
                content: { compiledJson: compiledJson, filePath: filePath },
            });
        });
    }
    sendCompletedCompileNotifToRust() {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            console.log("in sendCompletedCompileNotif");
            (_a = this._view) === null || _a === void 0 ? void 0 : _a.webview.postMessage({
                command: "completed_forge_build",
                content: {},
            });
        });
    }
    sendLostFocusToRust() {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            console.log("in sendLostFocusToRust");
            (_a = this._view) === null || _a === void 0 ? void 0 : _a.webview.postMessage({
                command: "lost_focus",
                content: {},
            });
        });
    }
    sendRegainedFocusToRust() {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            console.log("in sendRegainedFocusToRust");
            (_a = this._view) === null || _a === void 0 ? void 0 : _a.webview.postMessage({
                command: "gained_focus",
                content: {},
            });
        });
    }
    sendOpenOrClosedNotifToRust() {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            console.log("in sendFileOpenedNotifToRust");
            (_a = this._view) === null || _a === void 0 ? void 0 : _a.webview.postMessage({
                command: "file_opened_or_closed",
                content: {},
            });
        });
    }
    _getHtmlForWebview(webview) {
        const isDarkTheme = vscode.window.activeColorTheme.kind === vscode.ColorThemeKind.Dark ||
            vscode.window.activeColorTheme.kind === vscode.ColorThemeKind.HighContrast;
        const ethToolkitWasm = webview.asWebviewUri(vscode_1.Uri.joinPath(this._extensionUri, "wbg_out", "eth_toolkit_bg.wasm"));
        const ethToolkitJs = webview.asWebviewUri(vscode_1.Uri.joinPath(this._extensionUri, "wbg_out", "eth_toolkit.js"));
        const bridge = webview.asWebviewUri(vscode_1.Uri.joinPath(this._extensionUri, "wbg_out", "snippets", "eth_toolkit-eb19683a8f12ca3e", "src", "bridge.js"));
        // Use a nonce to only allow a specific script to be run.
        // const nonce = getNonce();
        //<meta http-equiv="Content-Security-Policy" content="default-src *; style-src 'unsafe-inline' *; script-src 'unsafe-inline' 'unsafe-eval' *; connect-src *;">
        // connect-src 'self' vscode-resource: http://127.0.0.1:8545 ">
        return `<!DOCTYPE html><html><head><meta http-equiv="Content-Type" content="text/html; charset=utf-8">

		<meta http-equiv="Content-Security-Policy" 
      content="default-src 'self';
               script-src 'self' 'unsafe-inline' 'unsafe-eval' vscode-resource: file: ;
               style-src 'self' 'unsafe-inline';
               connect-src *; ">



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
exports.ToolkitViewProvider = ToolkitViewProvider;
ToolkitViewProvider.viewType = "scope.toolkitView";
function getNonce() {
    let text = "";
    const possible = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    for (let i = 0; i < 32; i++) {
        text += possible.charAt(Math.floor(Math.random() * possible.length));
    }
    return text;
}
//# sourceMappingURL=extension.js.map