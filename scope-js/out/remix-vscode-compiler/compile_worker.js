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
const solc = require("solc");
const path = require("path");
const fs = require("fs");
const axios_1 = require("axios"); // can maybe remove? i think solcjs can  just handle this
const remix_resolver_1 = require("./remix_resolver");
function handleLocal(pathString, root) {
    console.log("handleLocal");
    console.log("pathstring: ", pathString, "root: ", root);
    try {
        const opts = { encoding: "utf8" };
        // hack for compiler imports to work (do not change)
        const p = path.resolve(root, pathString);
        const content = fs.readFileSync(p, opts);
        console.log(content);
        return { content, p };
    }
    catch (error) {
        // @ts-ignore
        process.send({ error });
        throw error;
    }
}
function findImports(path, root) {
    // TODO: We need current solc file path here for relative local import
    // @ts-ignore
    process.send({ processMessage: "importing file: " + path });
    const FSHandler = [
        {
            type: "local",
            match: (url) => {
                return /(^(?!(?:http:\/\/)|(?:https:\/\/)?(?:www.)?(?:github.com)))(^\/*[\w+-_/]*\/)*?(\w+\.sol)/g.exec(url);
            },
            handle: (match) => {
                console.log("findImports handle");
                return handleLocal(match[0], root);
            },
        },
    ];
    const urlResolver = new remix_resolver_1.RemixURLResolver();
    // this section usually executes after solc returns error file not found
    urlResolver
        .resolve(path, FSHandler)
        .then((data) => {
        console.log("data: %s", data);
        // @ts-ignore
        process.send({ data, path });
    })
        .catch((e) => {
        // @ts-ignore
        process.send({ error: e });
    });
    console.log("path: ", path, "root: ", root);
    return { error: "Deferred import" };
}
process.on("message", (m) => __awaiter(void 0, void 0, void 0, function* () {
    if (m.command === "compile") {
        const vnReg = /(^[0-9].[0-9].[0-9]\+commit\..*?)+(\.)/g;
        const vnRegArr = vnReg.exec(solc.version());
        // @ts-ignore
        const vn = "v" + (vnRegArr ? vnRegArr[1] : "");
        const input = m.payload;
        console.log(m);
        if (m.version === vn || m.version === "latest") {
            try {
                console.log("compiling with local version: ", solc.version());
                const output = yield solc.compile(JSON.stringify(input), { import: (path) => findImports(path, m.root) });
                // console.log(output);
                // @ts-ignore
                process.send({ compiled: output, version: solc.version() });
                // we should not exit process here as findImports still might be running
            }
            catch (e) {
                console.error(e);
                // @ts-ignore
                process.send({ error: e });
                // @ts-ignore
                process.exit(1);
            }
        }
        else {
            const v = m.version.replace("soljson-", "").replace(".js", "");
            console.log("Loading remote version " + v + "...");
            solc.loadRemoteVersion(v, (err, newSolc) => __awaiter(void 0, void 0, void 0, function* () {
                if (err) {
                    console.error(err);
                    // @ts-ignore
                    process.send({ error: err });
                }
                else {
                    console.log("compiling with remote version ", newSolc.version());
                    try {
                        const output = yield newSolc.compile(JSON.stringify(input), {
                            import: (path) => findImports(path, m.root),
                        });
                        // @ts-ignore
                        process.send({ compiled: output, version: newSolc.version() });
                    }
                    catch (e) {
                        console.error(e);
                        // @ts-ignore
                        process.send({ error: e });
                        // @ts-ignore
                        process.exit(1);
                    }
                }
            }));
        }
    }
    if (m.command === "fetch_compiler_verison") {
        axios_1.default
            .get("https://solc-bin.ethereum.org/bin/list.json")
            .then((res) => {
            // @ts-ignore
            process.send({ versions: res.data.releases });
        })
            .catch((e) => {
            // @ts-ignore
            process.send({ error: e });
            // @ts-ignore
            process.exit(1);
        });
    }
}));
//# sourceMappingURL=compile_worker.js.map