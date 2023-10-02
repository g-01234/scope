import * as wasm from "../../../eth_toolkit.js";

const vscode = acquireVsCodeApi();

export function send_message_to_extension() {
	console.log("in bridge.send_message_to_extension");
	vscode.postMessage({
		command: 'called_from_rust',
	});
}
export function get_open_files() {
	vscode.postMessage({
		command: 'get_open_files',
	})
}

export function get_file_contents(filePath) {
	vscode.postMessage({
		command: 'get_file_contents',
		data: filePath,
	})

}

export function test_fn() {
	return 1;
}

// handle messages sent from vscode -> bridge -> rust
window.addEventListener('message', event => {
	console.log("got message in bridge");
	const message = event.data;
	console.log(message);

	if (message.command === 'call_to_rust') {
		console.log(wasm.hello_from_rust());
	}
	if (message.command === 'post_open_files') {
		// console.log(wasm.hello_from_rust());
		wasm.receive_open_files(message.content);
	}
	if (message.command === 'post_file_contents') {
		wasm.receive_file_contents(message.content.data); // yuck
	}
});

console.log("loaded bridge")