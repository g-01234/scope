# scope 🔭

Scope is a VSCode extension for testing and interacting with smart contracts on the Ethereum blockchain. It is currently in early alpha and under active development. It is built on top of [Foundry](https://github.com/foundry-rs/), [ethers-rs](https://github.com/gakonst/ethers-rs) and [egui](https://github.com/emilk/egui); heavy inspiration is drawn from the [Remix project](https://github.com/ethereum/remix-project).

In many ways, Scope is a GUI wrapper for Foundry. It is intended to be used by developers and security researchers who are already familiar with the command-line tools and want a more visual way to interact with their contracts. Some currently supported features include:

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

1. Install the extension from the [VSCode Marketplace](https://marketplace.visualstudio.com/items?itemName=popular.scope-eth)
2. Ensure you have [foundry installed](https://book.getfoundry.sh/getting-started/installation) and run `foundryup`
3. Create a new foundry project via `foundry init scope-test`
4. Navigate to the root directory of the new project and open VSCode with `code .`
5. Open `src/Counter.sol` and `test/Counter.t.sol` in the VSCode editor window
6. Start a local Anvil instance via the `anvil` command (take care to not use the terminal window titled "scope"; the extension will use this terminal window for miscellaneous commands)

- You can also use `anvil -f <url>` to fork mainnet state.

### Basic Usage

- Scope requires VSCode to be opened in the root directory of a foundry project. It uses the foundry project's config (`foundry.toml`) to determine the solidity compiler version and other settings.
- The extension requires an Anvil node to be running at `localhost:8545` BEFORE the extension is activated (fix is TODO). If you start the node after opening VSCode, you will need to reload the window (`Command + Shift + P` -> "Developer: Reload Window")

1. Open `src/Counter.sol` and `test/Counter.t.sol` in the VSCode editor window
2. Click the telescope icon to open the extension
3. Press `Compile` to compile the contracts (this will run `forge build` in the background; Scope looks for compiled artifacts in `out/`)
4. Select `Counter.sol`
5. Click `Deploy` to deploy the contract to the local Anvil node
6. Select `Counter.t.sol` and run some tests (check the options as well)

Have some fun!

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
