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
exports.getTheme = exports.callTerminalHandleExit = exports.compileSoliditySource = exports.loadFile = exports.getCompiledForOpenFiles = void 0;
const vscode = require("vscode");
const vscode_1 = require("vscode");
function getCompiledForOpenFiles() {
    return __awaiter(this, void 0, void 0, function* () {
        let contracts = [];
        if (!vscode.workspace.workspaceFolders) {
            return contracts; // You may throw an error or log a message here
        }
        // Get the open .sol files
        const openFiles = vscode.window.tabGroups.all
            .flatMap(({ tabs }) => tabs.map((tab) => tab.label))
            .filter((filename) => filename.endsWith(".sol"));
        // Function to find and filter compiled contracts
        const findAndFilterCompiledContracts = (dirUri, contractName) => __awaiter(this, void 0, void 0, function* () {
            try {
                const compiledContracts = (yield vscode.workspace.fs.readDirectory(dirUri))
                    .filter(([name, type]) => type === vscode.FileType.File && name.endsWith(".json"))
                    .map(([name, _]) => vscode_1.Uri.joinPath(dirUri, name).toString());
                return compiledContracts;
            }
            catch (e) {
                return [];
            }
        });
        // Directories to check
        const directories = ["out", "artifacts/.foundry"];
        // Iterate over directories and collect compiled contracts for open .sol files
        for (const dir of directories) {
            for (const openFile of openFiles) {
                const dirUri = vscode_1.Uri.joinPath(vscode.workspace.workspaceFolders[0].uri, `${dir}/${openFile}`);
                const contractFiles = yield findAndFilterCompiledContracts(dirUri, openFile);
                contracts = [...new Set([...contracts, ...contractFiles])];
            }
        }
        return contracts;
    });
}
exports.getCompiledForOpenFiles = getCompiledForOpenFiles;
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
// async function findFilesWithExtension(
//   dirUri: vscode.Uri,
//   extension: string,
//   files: vscode.Uri[] = []
// ): Promise<vscode.Uri[]> {
//   const entries = await vscode.workspace.fs.readDirectory(dirUri);
//   for (const [name, type] of entries) {
//     const entryUri = dirUri.with({ path: dirUri.path + "/" + name });
//     if (type === vscode.FileType.Directory) {
//       await findFilesWithExtension(entryUri, extension, files);
//     } else if (type === vscode.FileType.File && name.endsWith(extension)) {
//       files.push(entryUri);
//     }
//   }
//   return files;
// }
// export async function getOpenSolidityFiles(): Promise<string[]> {
//   let files = [""];
//   if (vscode.workspace.workspaceFolders) {
//     const extension: string = ".sol";
//     // Get open files with .sol extension
//     const openFiles = vscode.window.tabGroups.all
//       .flatMap(({ tabs }) => tabs.map((tab) => tab.label))
//       .filter((filename) => filename.endsWith(extension));
//     // Function to find and filter files in a directory based on their extension
//     const findAndFilterFiles = async (dirUri: Uri, ext: string) => {
//       const pattern = new vscode.RelativePattern(dirUri, `**/*${ext}`);
//       const foundFiles = await vscode.workspace.findFiles(pattern, null, 100);
//       return foundFiles.filter((file) => openFiles.includes(path.basename(file.fsPath))).map((file) => file.fsPath);
//     };
//     // Check the root directory
//     // const rootDirUri: Uri = vscode.workspace.workspaceFolders[0].uri;
//     // const rootFiles = await findAndFilterFiles(rootDirUri, extension);
//     // Add any other directories you want to check here, for example:
//     const foundryNativeDir: Uri = Uri.joinPath(vscode.workspace.workspaceFolders[0].uri, "out");
//     const foundryNativeFiles = await findAndFilterFiles(foundryNativeDir, extension);
//     const hardhatDir: Uri = Uri.joinPath(vscode.workspace.workspaceFolders[0].uri, "artifacts/.foundry");
//     const hardhatFiles = await findAndFilterFiles(hardhatDir, extension);
//     // Combine the lists
//     files = [...new Set([...foundryNativeFiles, ...hardhatFiles])];
//   }
//   return files;
// }
// Note that this actually gets directory names; forge's `out` directory contains
// directories named after the compiled .sol files
// export async function getOpenCompiledFiles(): Promise<string[]> {
//   let files = [""];
//   if (vscode.workspace.workspaceFolders) {
//     // Get the open files
//     const openFiles = vscode.window.tabGroups.all
//       .flatMap(({ tabs }) => tabs.map((tab) => tab.label))
//       .filter((filename) => filename.endsWith(".sol"));
//     // Function to read a directory and filter its contents
//     const readAndFilterDirectory = async (dirPath: string) => {
//       //@ts-ignore - vscode.workspace.workspaceFolders[0] is not undefined
//       const targetDirUri: Uri = Uri.joinPath(vscode.workspace.workspaceFolders[0].uri, dirPath);
//       try {
//         const compiled = (await vscode.workspace.fs.readDirectory(targetDirUri))
//           .filter(([name, _]) => openFiles.includes(name))
//           .map(([name, _]) => name);
//         return compiled;
//       } catch {
//         return [];
//       }
//     };
//     // Check the ./out/ directory
//     const foundryNativeFiles = await readAndFilterDirectory("out");
//     // Check the ./artifacts/.foundry/ directory (for hardhat-based projects)
//     const hardhatFiles = await readAndFilterDirectory("artifacts/.foundry");
//     // Combine the lists
//     files = [...new Set([...foundryNativeFiles, ...hardhatFiles])];
//     console.log("Files in getOpenCompiledFiles: ", files);
//   }
//   return files;
// }
// export async function getOpenCompiledContracts(): Promise<string[]> {
//   let contracts: string[] = [];
//   if (!vscode.workspace.workspaceFolders) {
//     return contracts; // should probably throw an error
//   }
//   // Function to find and filter compiled contract files in a directory
//   const findAndFilterCompiledContracts = async (dirUri: Uri) => {
//     try {
//       const compiledContracts = (await vscode.workspace.fs.readDirectory(dirUri))
//         .filter(([name, type]) => type === vscode.FileType.File && name.endsWith(".json"))
//         .map(([name, _]) => Uri.joinPath(dirUri, name).toString());
//       return compiledContracts;
//     } catch (e) {
//       return [];
//     }
//   };
//   // List of directories you want to check
//   const directories = ["out", "artifacts/.foundry"];
//   // Iterate over the directories and collect the compiled contracts
//   for (const dir of directories) {
//     const dirUri: Uri = Uri.joinPath(vscode.workspace.workspaceFolders[0].uri, dir);
//     console.log("dirUri: ", dirUri);
//     const contractFiles = await findAndFilterCompiledContracts(dirUri);
//     console.log(contractFiles);
//     contracts = [...new Set([...contracts, ...contractFiles])];
//   }
//   console.log("Contracts in getOpenCompiledContracts: ", contracts);
//   return contracts;
// }
//# sourceMappingURL=helpers.js.map