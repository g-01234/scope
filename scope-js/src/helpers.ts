import * as vscode from "vscode";
import { commands, ExtensionContext, Uri, Webview, WebviewView, WebviewViewProvider, window, workspace } from "vscode";
let path = require("path");
let fs = require("fs");

export async function getOpenSolidityFiles(): Promise<string[]> {
  let files = [""];
  if (vscode.workspace.workspaceFolders) {
    const targetDirUri: Uri = vscode.workspace.workspaceFolders[0].uri; // Adjust as needed
    const extension: string = ".sol"; // Adjust as needed
    const openFiles = vscode.window.tabGroups.all
      .flatMap(({ tabs }) => tabs.map((tab) => tab.label))
      .filter((filename) => filename.endsWith(".sol"));

    files = (await findFilesWithExtension(targetDirUri, extension))
      .filter((file) => {
        return openFiles.includes(path.basename(file.fsPath));
      })
      .map((file) => {
        return file.fsPath;
      });
  }
  return files;
}

// Note that this actually gets directory names; forge's `out` directory contains
// directories named after the compiled .sol files
export async function getOpenCompiledFiles(): Promise<string[]> {
  let files = [""];
  if (vscode.workspace.workspaceFolders) {
    const targetDirUri: Uri = Uri.joinPath(vscode.workspace.workspaceFolders[0].uri, "out"); // Adjust as needed
    const openFiles = vscode.window.tabGroups.all
      .flatMap(({ tabs }) => tabs.map((tab) => tab.label))
      .filter((filename) => filename.endsWith(".sol"));

    // If the ./out/ directory doesn't exist (i.e. we haven't 📋compiled), want to just return [""]
    try {
      const compiled = (await vscode.workspace.fs.readDirectory(targetDirUri))
        .filter(([name, _]) => {
          return openFiles.includes(name);
        })
        .map(([name, _]) => {
          return name;
        });
      files = compiled;
    } catch {}
  }
  return files;
}

export async function getOpenCompiledContracts(directories: string[], outDir: Uri): Promise<string[]> {
  let contracts: string[] = [];
  for (const dir of directories) {
    const dirUri: Uri = Uri.joinPath(outDir, dir);
    const contractFiles = (await vscode.workspace.fs.readDirectory(dirUri))
      .filter(([name, type]) => type === vscode.FileType.File && name.endsWith(".json"))
      .map(([name, _]) => Uri.joinPath(dirUri, name).toString());
    contracts = contracts.concat(contractFiles);
  }
  return contracts;
}

async function findFilesWithExtension(
  dirUri: vscode.Uri,
  extension: string,
  files: vscode.Uri[] = []
): Promise<vscode.Uri[]> {
  const entries = await vscode.workspace.fs.readDirectory(dirUri);
  for (const [name, type] of entries) {
    const entryUri = dirUri.with({ path: dirUri.path + "/" + name });
    if (type === vscode.FileType.Directory) {
      await findFilesWithExtension(entryUri, extension, files);
    } else if (type === vscode.FileType.File && name.endsWith(extension)) {
      files.push(entryUri);
    }
  }
  return files;
}

export async function loadFile(uri: vscode.Uri): Promise<Uint8Array> {
  return await vscode.workspace.fs.readFile(uri);
}

export async function compileSoliditySource(soliditySource: string) {
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
}

export async function callTerminalHandleExit(command: string) {
  console.log("in callTerminalHandleExit");
  const tempTerminal = vscode.window.createTerminal({
    location: vscode.TerminalLocation.Panel,
    name: "burner",
  });
  tempTerminal.show();
  tempTerminal.sendText(command, false);
  tempTerminal.sendText("; exit");
  return new Promise((resolve, reject) => {
    const disposeToken = vscode.window.onDidCloseTerminal(async (closedTerminal) => {
      if (closedTerminal === tempTerminal) {
        // Want to handle this better eventually
        disposeToken.dispose();
        if (tempTerminal.exitStatus !== undefined) {
          if (tempTerminal.exitStatus.code !== 0) {
            vscode.window.withProgress(
              { location: vscode.ProgressLocation.Notification, cancellable: true },
              (progress) => {
                progress.report({
                  message:
                    "Error with compilation; try running `forge build` in your terminal and then pressing the 'Compile' button again in the extension.",
                });
                return new Promise<void>((resolve) => {
                  setTimeout(() => {
                    resolve();
                  }, 10000);
                });
              }
            );
          }
          resolve(tempTerminal.exitStatus);
        } else {
          reject("Terminal exited with undefined status");
        }
      }
    });
  });
}

export async function getTheme() {
  const theme = vscode.workspace.getConfiguration("workbench").get("panel.background");
  const color = new vscode.ThemeColor("activityBar.background");
  console.log("Theme: ", color);
  return theme;
}
