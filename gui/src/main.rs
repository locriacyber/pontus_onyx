#![allow(clippy::needless_return)]
#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

use std::convert::From;
use std::sync::{Arc, Mutex};

fn main() {
	tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![greet])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
	let result = format!("Hello, {}!", name);

	result
}
