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
const vscode = require("vscode");
const child_process_1 = require("child_process");
const path = require("path");
// import { relativePath } from "@remixproject/engine-vscode/util/path";
const profile = {
    name: "solidity",
    displayName: "Solidity compiler",
    description: "Compile solidity contracts",
    kind: "compiler",
    permission: true,
    location: "sidePanel",
    documentation: "https://remix-ide.readthedocs.io/en/latest/solidity_editor.html",
    version: "0.0.1",
    methods: ["getCompilationResult", "compile", "compileWithParameters", "setCompilerConfig"],
};
class NativeSolcPlugin {
    constructor() {
        this.version = "0.0.1";
        this.versions = [];
        // super(profile);
        this.outputChannel = vscode.window.createOutputChannel("Remix IDE");
        this.compilationResult = {
            source: {
                target: "",
                sources: {},
            },
            data: null,
        };
        this.loadSolidityVersions();
    }
    getVersion() {
        return this.version;
    }
    createWorker() {
        // enable --inspect for debug
        // return fork(path.join(__dirname, "compile_worker.js"), [], {
        //   execArgv: ["--inspect=" + (process.debugPort + 1)]
        // });
        return (0, child_process_1.fork)(path.join(__dirname, "compile_worker.js"));
    }
    getNow() {
        const date = new Date(Date.now());
        return date.toLocaleTimeString();
    }
    print(m) {
        const now = this.getNow();
        this.outputChannel.appendLine(`[${now}]: ${m}`);
        this.outputChannel.show();
    }
    compile(_version, filePath, opts) {
        return __awaiter(this, void 0, void 0, function* () {
            this.print("Compilation started!");
            this.version = _version in this.versions ? this.versions[_version] : _version;
            // const fileName = await this.call("fileManager", "getCurrentFile");
            // this.print(`Compiling ${fileName} ...`);
            // const editorContent = window.activeTextEditor ? window.activeTextEditor.document.getText() : undefined;
            const sources = {};
            // if (fileName) {
            const fileName = path.basename(filePath);
            sources[fileName] = {
                content: (yield vscode.workspace.fs.readFile(vscode.Uri.parse(filePath))).toString(),
            };
            // }
            const solcWorker = this.createWorker();
            console.log(`Solidity compiler invoked with WorkerID: ${solcWorker.pid}`);
            console.log(`Compiling with solidity version ${this.version}`);
            var input = {
                language: opts.language || "Solidity",
                sources,
                settings: {
                    outputSelection: {
                        "*": {
                            "": ["ast"],
                            "*": [
                                "abi",
                                "metadata",
                                "devdoc",
                                "userdoc",
                                "evm.legacyAssembly",
                                "evm.bytecode",
                                "evm.deployedBytecode",
                                "evm.methodIdentifiers",
                                "evm.gasEstimates",
                                "evm.assembly",
                            ],
                        },
                    },
                    optimizer: {
                        enabled: opts.optimize === true || opts.optimize === 1,
                        runs: opts.runs || 200,
                        details: {
                            yul: Boolean(opts.language === "Yul" && opts.optimize),
                        },
                    },
                    libraries: opts.libraries,
                },
            };
            if (opts.evmVersion) {
                input.settings.evmVersion = opts.evmVersion;
            }
            if (opts.language) {
                input.language = opts.language;
            }
            if (opts.language === "Yul" && input.settings.optimizer.enabled) {
                if (!input.settings.optimizer.details)
                    input.settings.optimizer.details = {};
                input.settings.optimizer.details.yul = true;
            }
            // typescript cope
            if (!vscode.workspace.workspaceFolders) {
                throw new Error("No workspace open");
            }
            const root = vscode.Uri.joinPath(vscode.workspace.workspaceFolders[0].uri, "src/").fsPath;
            solcWorker.send({
                command: "compile",
                root: root,
                payload: input,
                version: this.version,
            });
            return new Promise((resolve, reject) => {
                solcWorker.on("message", (m) => {
                    console.log(`............................Solidity worker message............................`);
                    console.log(m);
                    if (m.error) {
                        this.print(m.error);
                        console.error(m.error);
                    }
                    else if (m.data && m.path) {
                        sources[m.path] = {
                            content: m.data.content,
                        };
                        solcWorker.send({
                            command: "compile",
                            root: root,
                            payload: input,
                            version: this.version,
                        });
                    }
                    else if (m.compiled) {
                        const languageVersion = this.version;
                        const compiled = JSON.parse(m.compiled);
                        if (compiled.errors) {
                            // this.print("here"); //delete
                            this.print(`Compilation error while compiling ${fileName} with solidity version ${m === null || m === void 0 ? void 0 : m.version}.`);
                            logError(compiled === null || compiled === void 0 ? void 0 : compiled.errors);
                        }
                        if (compiled.contracts) {
                            // const source = { sources };
                            const data = JSON.parse(m.compiled);
                            this.compilationResult = {
                                source: {
                                    sources,
                                    target: fileName,
                                },
                                data,
                            };
                            this.print(`Compilation finished for ${fileName} with solidity version ${m === null || m === void 0 ? void 0 : m.version}.`);
                            // this.emit("compilationFinished", fileName, source, languageVersion, data);
                            resolve(this.compilationResult);
                        }
                    }
                });
                const errorKeysToLog = ["formattedMessage"];
                const logError = (errors) => {
                    for (let i in errors) {
                        if (["number", "string"].includes(typeof errors[i])) {
                            if (errorKeysToLog.includes(i))
                                this.print(errors[i]);
                        }
                        else {
                            logError(errors[i]);
                        }
                    }
                };
            });
        });
    }
    // async compileWithSolidityExtension() {
    //   commands.executeCommand("solidity.compile.active").then(async (listOFiles: string[]) => {
    //     if (listOFiles)
    //       for (let file of listOFiles) {
    //         await this.parseSolcOutputFile(file)
    //       }
    //   })
    // }
    // async parseSolcOutputFile(file: string) {
    //   console.log(file)
    //   this.print(`Compiling with Solidity Extension`)
    //   const content = await this.call("fileManager", "readFile", file)
    //   const parsedContent = JSON.parse(content)
    //   const sourcePath = parsedContent.sourcePath
    //   const solcOutput = `${path.basename(parsedContent.sourcePath).split('.').slice(0, -1).join('.')}-solc-output.json`
    //   const outputDir = path.dirname(file)
    //   let raw = await this.call("fileManager", "readFile", `${outputDir}/${solcOutput}`)
    //   console.log(`${outputDir}/${solcOutput}`);
    //   const relativeFilePath = relativePath(sourcePath)
    //   var re = new RegExp(`${sourcePath}`, "gi");
    //   raw = raw.replace(re, relativeFilePath)
    //   const compiled = JSON.parse(raw)
    //   let source = {}
    //   const fileKeys = Object.keys(compiled.sources)
    //   for (let s of fileKeys) {
    //     source[s] = { content: await this.call("fileManager", "readFile", s) }
    //   }
    //   this.compilationResult = {
    //     source: {
    //       sources: source,
    //       target: relativeFilePath
    //     },
    //     data: compiled
    //   }
    //   this.print(`Compilation finished for ${relativeFilePath} with solidity version ${parsedContent?.compiler.version}.`);
    //   this.emit('compilationFinished', relativeFilePath, { sources: source }, parsedContent?.compiler.version, compiled);
    // }
    getCompilationResult() {
        return this.compilationResult;
    }
    loadSolidityVersions() {
        const solcWorker = this.createWorker();
        solcWorker.send({ command: "fetch_compiler_verison" });
        solcWorker.on("message", (m) => {
            this.versions = m.versions;
        });
    }
    getSolidityVersions() {
        return this.versions;
    }
}
exports.default = NativeSolcPlugin;
//# sourceMappingURL=native_solidity_plugin.js.map