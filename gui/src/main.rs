#![allow(clippy::needless_return)]
#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

fn main() {
	tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![init_gui])
		.invoke_handler(tauri::generate_handler![install_server])
		.invoke_handler(tauri::generate_handler![start_server])
		.invoke_handler(tauri::generate_handler![stop_server])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}

#[tauri::command]
fn init_gui() -> (Vec<String>) {
	todo!()
}

#[tauri::command]
fn install_server(install_path: String, username: String, password: String) -> Result<String, String> {
	todo!()
}

#[tauri::command]
fn start_server() {
	todo!()
}

#[tauri::command]
fn stop_server() {
	todo!()
}
