pub fn load_or_create_settings(settings_path: std::path::PathBuf) -> Settings {
	println!(
		"\t📢 Trying to read settings file `{}`",
		&settings_path.to_str().unwrap_or_default()
	);
	println!();

	let settings = match std::fs::read(&settings_path) {
		Ok(bytes) => match toml::from_slice(&bytes) {
			Ok(settings) => settings,
			Err(e) => {
				println!("\t⚠ Can not parse settings file : {}", e);
				println!("\t✔ Falling back to default settings");

				Settings::default()
			}
		},
		Err(e) => {
			println!("\t⚠ Can not read settings file : {}", e);

			let result = Settings::default();

			if e.kind() == std::io::ErrorKind::NotFound {
				if let Some(parent) = settings_path.parent() {
					if let Err(e) = std::fs::create_dir_all(parent) {
						println!(
							"\t\t❌ Can not creating parent folders of settings file : {}",
							e
						);
					}
				}

				match std::fs::write(settings_path, toml::to_vec(&result).unwrap()) {
					Ok(_) => {
						println!("\t\t✔ Creating default settings file.");
					}
					Err(e) => {
						println!("\t\t❌ Can not creating default settings file : {}", e);
					}
				}
			}

			println!("\t\t✔ Falling back to default settings.");
			println!();

			result
		}
	};

	settings
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Settings {
	pub port: usize,
	pub admin_email: String,
	pub token_lifetime_seconds: u64,
	pub logfile_path: String,
	pub userfile_path: String,
	pub data_path: String,
	pub https: SettingsHTTPS,
}
impl Default for Settings {
	fn default() -> Self {
		Self {
			port: 7541,
			admin_email: String::from(""),
			token_lifetime_seconds: 60 * 60,
			logfile_path: String::from("database/logs.msgpack"),
			userfile_path: String::from("database/users.bin"),
			data_path: String::from("database/data"),
			https: SettingsHTTPS::default(),
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SettingsHTTPS {
	pub port: usize,
	pub keyfile_path: String,
	pub certfile_path: String,
	pub enable_hsts: bool,
}
impl Default for SettingsHTTPS {
	fn default() -> Self {
		Self {
			port: 7542,
			keyfile_path: String::new(),
			certfile_path: String::new(),
			enable_hsts: true,
		}
	}
}
