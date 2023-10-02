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
exports.getTheme = exports.callTerminalHandleExit = exports.compileSoliditySource = exports.loadFile = exports.getOpenCompiledContracts = exports.getOpenCompiledFiles = exports.getOpenSolidityFiles = void 0;
const vscode = require("vscode");
const vscode_1 = require("vscode");
let path = require("path");
let fs = require("fs");
function getOpenSolidityFiles() {
    return __awaiter(this, void 0, void 0, function* () {
        let files = [""];
        if (vscode.workspace.workspaceFolders) {
            const targetDirUri = vscode.workspace.workspaceFolders[0].uri; // Adjust as needed
            const extension = ".sol"; // Adjust as needed
            const openFiles = vscode.window.tabGroups.all
                .flatMap(({ tabs }) => tabs.map((tab) => tab.label))
                .filter((filename) => filename.endsWith(".sol"));
            files = (yield findFilesWithExtension(targetDirUri, extension))
                .filter((file) => {
                return openFiles.includes(path.basename(file.fsPath));
            })
                .map((file) => {
                return file.fsPath;
            });
        }
        return files;
    });
}
exports.getOpenSolidityFiles = getOpenSolidityFiles;
// Note that this actually gets directory names; forge's `out` directory contains
// directories named after the compiled .sol files
function getOpenCompiledFiles() {
    return __awaiter(this, void 0, void 0, function* () {
        let files = [""];
        if (vscode.workspace.workspaceFolders) {
            const targetDirUri = vscode_1.Uri.joinPath(vscode.workspace.workspaceFolders[0].uri, "out"); // Adjust as needed
            const openFiles = vscode.window.tabGroups.all
                .flatMap(({ tabs }) => tabs.map((tab) => tab.label))
                .filter((filename) => filename.endsWith(".sol"));
            // If the ./out/ directory doesn't exist (i.e. we haven't 📋compiled), want to just return [""]
            try {
                const compiled = (yield vscode.workspace.fs.readDirectory(targetDirUri))
                    .filter(([name, _]) => {
                    return openFiles.includes(name);
                })
                    .map(([name, _]) => {
                    return name;
                });
                files = compiled;
            }
            catch (_a) { }
        }
        return files;
    });
}
exports.getOpenCompiledFiles = getOpenCompiledFiles;
function getOpenCompiledContracts(directories, outDir) {
    return __awaiter(this, void 0, void 0, function* () {
        let contracts = [];
        for (const dir of directories) {
            const dirUri = vscode_1.Uri.joinPath(outDir, dir);
            const contractFiles = (yield vscode.workspace.fs.readDirectory(dirUri))
                .filter(([name, type]) => type === vscode.FileType.File && name.endsWith(".json"))
                .map(([name, _]) => vscode_1.Uri.joinPath(dirUri, name).toString());
            contracts = contracts.concat(contractFiles);
        }
        return contracts;
    });
}
exports.getOpenCompiledContracts = getOpenCompiledContracts;
function findFilesWithExtension(dirUri, extension, files = []) {
    return __awaiter(this, void 0, void 0, function* () {
        const entries = yield vscode.workspace.fs.readDirectory(dirUri);
        for (const [name, type] of entries) {
            const entryUri = dirUri.with({ path: dirUri.path + "/" + name });
            if (type === vscode.FileType.Directory) {
                yield findFilesWithExtension(entryUri, extension, files);
            }
            else if (type === vscode.FileType.File && name.endsWith(extension)) {
                files.push(entryUri);
            }
        }
        return files;
    });
}
function loadFile(uri) {
    return __awaiter(this, void 0, void 0, function* () {
        return yield vscode.workspace.fs.readFile(uri);
    });
}
exports.loadFile = loadFile;
function compileSoliditySource(soliditySource) {
    return __awaiter(this, void 0, void 0, function* () {
        var input = {
            language: "Solidity",
            sources: {
                "test.sol": {
                    content: soliditySource,
                },
            },
            settings: {
                outputSelection: {
                    "*": {
                        "*": ["*"],
                    },
                },
            },
        };
        // // let compiled = JSON.parse(solc.compile(JSON.stringify(input)));
        // console.log(compiled);
        // return JSON.stringify(compiled.contracts);
    });
}
exports.compileSoliditySource = compileSoliditySource;
function callTerminalHandleExit(command) {
    return __awaiter(this, void 0, void 0, function* () {
        console.log("in callTerminalHandleExit");
        const tempTerminal = vscode.window.createTerminal({
            location: vscode.TerminalLocation.Panel,
            name: "burner",
        });
        tempTerminal.show();
        tempTerminal.sendText(command, false);
        tempTerminal.sendText("; exit");
        return new Promise((resolve, reject) => {
            const disposeToken = vscode.window.onDidCloseTerminal((closedTerminal) => __awaiter(this, void 0, void 0, function* () {
                if (closedTerminal === tempTerminal) {
                    // Want to handle this better eventually
                    disposeToken.dispose();
                    if (tempTerminal.exitStatus !== undefined) {
                        if (tempTerminal.exitStatus.code !== 0) {
                            vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, cancellable: true }, (progress) => {
                                progress.report({
                                    message: "Error with compilation; try running `forge build` in your terminal and then pressing the 'Compile' button again in the extension.",
                                });
                                return new Promise((resolve) => {
                                    setTimeout(() => {
                                        resolve();
                                    }, 10000);
                                });
                            });
                        }
                        resolve(tempTerminal.exitStatus);
                    }
                    else {
                        reject("Terminal exited with undefined status");
                    }
                }
            }));
        });
    });
}
exports.callTerminalHandleExit = callTerminalHandleExit;
function getTheme() {
    return __awaiter(this, void 0, void 0, function* () {
        const theme = vscode.workspace.getConfiguration("workbench").get("panel.background");
        const color = new vscode.ThemeColor("activityBar.background");
        console.log("Theme: ", color);
        return theme;
    });
}
exports.getTheme = getTheme;
//# sourceMappingURL=helpers.js.map