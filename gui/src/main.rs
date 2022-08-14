#![allow(clippy::needless_return)]
#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

use std::sync::{Arc, Mutex};

fn main() {
	std::panic::set_hook(Box::new(|err| {
		let mut error = format!(
			"{} V{}\n\n{err}",
			env!("CARGO_PKG_NAME").to_uppercase(),
			env!("CARGO_PKG_VERSION")
		)
		.replace("'", "")
		.replace("\"", "");

		match err.payload().downcast_ref::<&str>() {
			Some(err) => {
				if !error.is_empty() {
					error += "\n\n";
				}
				error += &err.replace("'", "").replace("\"", "");
			}
			None => {
				if error.is_empty() {
					error += "(panic without payload)";
				}
			}
		}

		tinyfiledialogs::message_box_ok(
			&format!("{} : Fatal error", env!("CARGO_PKG_NAME").to_uppercase()),
			&error,
			tinyfiledialogs::MessageBoxIcon::Error,
		);
	}));

	println!(
		"{} V{}",
		env!("CARGO_PKG_NAME").to_uppercase(),
		env!("CARGO_PKG_VERSION")
	);
	println!();

	let workspace_path =
		std::path::PathBuf::from(if let Some(workspace_dir) = std::env::args().nth(1) {
			if let Err(err) = std::fs::create_dir_all(workspace_dir.clone()) {
				panic!(
					"Error : can not create workspace {} : {}",
					workspace_dir, err
				);
			}

			workspace_dir
		} else {
			String::from("database")
		});

	let temp_logs_list = Arc::new(Mutex::new(vec![]));
	let temp_logs_list_for_dispatcher = temp_logs_list.clone();
	let mut temp_logger = charlie_buffalo::Logger::new(
		charlie_buffalo::new_dispatcher(Box::from(move |log: charlie_buffalo::Log| {
			temp_logs_list_for_dispatcher.lock().unwrap().push(log);
		})),
		charlie_buffalo::new_dropper(Box::from(|_: &charlie_buffalo::Logger| {})),
	);

	temp_logger.push(
		vec![
			(String::from("event"), String::from("setup")),
			(String::from("level"), String::from("INFO")),
		],
		Some("setup of the program"),
	);

	let mut settings_path = workspace_path.clone();
	settings_path.push("settings.toml");
	let settings =
		pontus_onyx::http_server::load_or_create_settings(settings_path.clone(), &mut temp_logger);

	let mut ports = vec![settings.port];
	if let Some(ref https) = settings.https {
		ports = vec![https.port, settings.port];
	}

	let logger =
		pontus_onyx::http_server::load_or_create_logger(&settings, temp_logger, temp_logs_list);

	let db_users = pontus_onyx::http_server::load_or_create_users(&settings, logger.clone());
	let users = db_users.get_usernames().into_iter().cloned().collect();

	let localhost = String::from("localhost");

	tauri::Builder::default()
		.manage(AppState {
			working_folder: format!("{}", dunce::canonicalize(workspace_path).unwrap().display()),
			domain: settings.domain.as_ref().unwrap_or_else(|| &localhost).clone(),
			users,
			ports,
			status: ServerStatus::Disabled, // TODO
		})
		.invoke_handler(tauri::generate_handler![
			init_gui,
			install_server,
			start_server,
			stop_server
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}

#[derive(serde::Serialize)]
struct AppState {
	working_folder: String,
	domain: String,
	users: Vec<String>,
	ports: Vec<usize>,
	status: ServerStatus,
}

#[derive(serde::Serialize)]
enum ServerStatus {
	Enabled,
	Disabled,
	Uninstalled,
}

#[tauri::command]
fn init_gui(state: tauri::State<AppState>) -> String {
	let state = state.inner();
	serde_json::to_string(&state).unwrap()
}

#[tauri::command]
fn install_server(
	install_path: String,
	username: String,
	password: String,
) -> Result<String, String> {
	todo!()
}

#[tauri::command]
fn start_server(state: tauri::State<AppState>) {
	todo!()
}

#[tauri::command]
fn stop_server() {
	todo!()
}
