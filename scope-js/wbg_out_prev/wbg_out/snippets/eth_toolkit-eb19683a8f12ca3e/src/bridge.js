import * as wasm from "../../../eth_toolkit.js";

const vscode = acquireVsCodeApi();

// rust calls these exported fns to send messages to vscode
// rust -> bridge -> vscode

export function get_vscode_style() {
  const style = getComputedStyle(document.documentElement);

  const exported_style = JSON.stringify({
    isDarkTheme: isDarkTheme,
    "editor.background": style.getPropertyValue("--vscode-editor-background"),
    "editor.foreground": style.getPropertyValue("--vscode-editor-foreground"),
    "activityBar.background": style.getPropertyValue("--vscode-activityBar-background"),
    "activityBar.activeBorder": style.getPropertyValue("--vscode-activityBar-activeBorder"),
  });

  // console.log("exported style", exported_style);
  return exported_style;
}

export function get_open_files() {
  console.log("about to post get_open_files to vscode");
  vscode.postMessage({
    command: "get_open_files",
  });
}
export function get_file_contents(filePath) {
  vscode.postMessage({
    command: "get_file_contents",
    data: { filePath: filePath },
  });
}
export function get_compiled_solidity(filePath) {
  vscode.postMessage({
    command: "get_compiled_solidity",
    data: { filePath: filePath },
  });
}
export function forge_build(command) {
  vscode.postMessage({
    command: "forge_build",
    data: { command: command },
  });
}
export function execute_shell_command(command) {
  vscode.postMessage({
    command: "execute_shell_command",
    data: { command: command },
  });
}
export function send_error_to_vscode(errorText) {
  vscode.postMessage({
    command: "error_popup",
    data: { text: errorText },
  });
}
export function send_ok_to_vscode(okText) {
  vscode.postMessage({
    command: "ok_popup",
    data: { text: okText },
  });
}

window.addEventListener("blur", () => {
  vscode.postMessage({ command: "webviewBlurred" });
});

window.addEventListener("focus", (event) => {
  vscode.postMessage({ command: "webviewFocused" });
});

document.addEventListener("DOMContentLoaded", function () {});

// this part handles messages sent from vscode -> bridge -> rust
window.addEventListener("message", (event) => {
  console.log("got message in bridge");
  const message = event.data;
  console.log(message);

  switch (message.command) {
    case "call_to_rust":
      console.log(wasm.hello_from_rust());
      break;

    case "post_open_file_paths":
      wasm.receive_open_file_paths(message.content);
      break;

    case "post_file_contents":
      wasm.receive_file_contents(message.content.fileContents); // yuck
      break;

    case "post_compiled_solidity":
      wasm.receive_compiled_solidity(message.content.compiledJson, message.content.filePath);
      break;

    case "completed_forge_build":
      wasm.handle_completed_forge_build();
      break;
    case "lost_focus":
      wasm.handle_losing_focus();
      break;
    case "gained_focus":
      wasm.handle_gaining_focus();
      break;

    // just refreshes the file list
    case "file_opened_or_closed":
      wasm.handle_file_opened_or_closed();
      break;

    default:
      console.log("Unrecognized command:", message.command);
      break;
  }
});

console.log("loaded bridge");
