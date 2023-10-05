# scope 🔭

Scope is a VSCode extension for testing and interacting with smart contracts on the Ethereum blockchain. It is currently in early alpha and under active development. It is built on top of [Foundry](https://github.com/foundry-rs/), [ethers-rs](https://github.com/gakonst/ethers-rs) and [egui](https://github.com/emilk/egui); heavy inspiration is drawn from the [Remix project](https://github.com/ethereum/remix-project).

In many ways, Scope aims to be a GUI wrapper for Foundry. It is intended to be used by developers and security researchers who are already familiar with the command-line tools and want a more visual way to interact with their contracts/tests. Some currently supported features include:

- Deploying contracts against a local Anvil node (or a mainnet fork via `anvil -f <url>`)
- Loading existing contracts at an address
- Interacting with contracts either via ABI or raw calldata
- Easy copying of ABI, AST, calldata, return values, etc.
- Setting storage slots and balances
- One-button printing storage layout and contract interfaces via `cast`
- Debugging transaction traces via `cast call --trace` + forge debugger
- Running foundry tests
- Misc. CLI wrappers:
  - forge build
  - slither
  - pyrometer
  - chisel

(see TODO.md for a scratchpad of planned features)

## Quickstart

### Installation

If you run into any issues, DM me on twitter @popular_12345 and I'll help you out. Installation + setup can be a little finnicky currently but I'm aiming to streamline it.

1. Install the extension from the [VSCode Marketplace](https://marketplace.visualstudio.com/items?itemName=popular.scope-eth)
2. Ensure you have [foundry installed](https://book.getfoundry.sh/getting-started/installation) and run `foundryup`
3. Start a local Anvil instance via the `anvil` command (take care to not use the terminal window titled "scope"; the extension will use this terminal window for miscellaneous commands). I'd recommend doing this in an external terminal (in case you have to reload VSCode)
   - You can also use `anvil -f <url>` to fork mainnet state.

### Basic Usage

- _NOTE1_ - Scope requires VSCode to be opened in the root directory of a foundry project. It piggybacks on the foundry compiler / command line commands
- _NOTE2_ - The extension requires an Anvil node to be running at `localhost:8545` BEFORE the extension is activated (it performs some node setup on load - `hardhat_autoImpersonateAccount` and `hardhat_setBalance`). If you start the node after opening VSCode, you will need to reload the extension (`Command + Shift + P` -> "Developer: Reload Webviews")

1. Create a new foundry project via `foundry init scope-test` (or open VSCode at the root of an existing one)
2. Navigate to the root directory of the new project and open VSCode with `code .`
3. Open `src/Counter.sol` and `test/Counter.t.sol` in the VSCode editor window
4. Click the telescope icon to open the extension
5. Press `Compile` to compile the contracts (this will run `forge build` in the background; Scope looks for compiled artifacts in `out/`)

   - Alternately, run "forge build" in the terminal and press the "Refresh" button at the top of the extension

6. Select `Counter.sol`
7. Click `Deploy` to deploy the contract to the local Anvil node
8. (Separate) Select `Counter.t.sol` and run some tests (check the options as well)

### Troubleshooting

Most issues can be resolved with some combination of the following:

- Ensure you have an anvil node running at `localhost:8545` (the default)
- Ensure you have VSCode opened at the root of a foundry project
- Reload VSCode or the extension webview - `Command + Shift + P` -> "Developer: Reload Window/Webviews"
- Run `forge clean && forge build` in the terminal -> press the "🔄" button at the top of the extension after successful compilation to refresh the contract list (the `Compile` button can be error-prone)

## Development Quickstart (test/dev)

1. Clone the repo
2. Ensure you have [foundry installed](https://book.getfoundry.sh/getting-started/installation) and run `foundryup`
3. Start an anvil node (using ethers-rs default): `anvil -m "stuff inherit faith park genre spread huge knee ecology private marble supreme"`
4. Open VSCode with `scope-js` as the root directory and press `Fn + F5`.
5. Open `./example` as the root directory of the newly-opened `Extension Development Host`
   - The extension should be installed, but may need to be pinned in order to remain visible
6. Open `./example/src/Sample.sol` in VSCode
   - Press "Compile"
   - You may need to press the `🔄` button; handling active opening+closing of files is WIP
7. Deploy, etc!

## Repo Contents:

- scope-js - VSCode extension logic
  - Takes a rust wasm binary and serves it in a VSCode Webview View
  - Handles:
    - Extension config
    - Loading the extension into VSCode
    - File system access
    - Packaging whole extension to VSIX binary (via @vscode/vsce) for easy installation
- scope-rs - Rust egui application
  - Handles:
    - Rendering the immediate mode UI (egui claims ~60hz but I'm not sure)
    - Application logic
    - Compiling to wasm (`scope-rs/build.sh`)

## Building:

- To compile rust to wasm, run `./build.sh` from within the scope-rs directory
  - This also copies the wasm + bindings to `scope-js/wbg_out`, which is where the extension knows to look
- To run the extension in dev mode, open vscode (or a VSCode fork) with `scope-js` as the root directory and press `Fn + F5`
- To add the extension to your live VSCode, package it using [@vscode/vsce](https://github.com/microsoft/vscode-vsce) (`$ vsce package`) and install the extension from VSIX
