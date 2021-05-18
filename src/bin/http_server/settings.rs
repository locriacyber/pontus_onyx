pub fn load_or_create_settings(
	settings_path: std::path::PathBuf,
	logger: &mut charlie_buffalo::Logger,
) -> Settings {
	let settings = match std::fs::read(&settings_path) {
		Ok(bytes) => match toml::from_slice(&bytes) {
			Ok(settings) => {
				logger.push(
					vec![
						(String::from("event"), String::from("setup")),
						(String::from("module"), String::from("settings")),
						(String::from("level"), String::from("INFO")),
					],
					Some(&format!(
						"settings successfully loaded from `{}`",
						&settings_path.to_str().unwrap_or_default()
					)),
				);

				settings
			}
			Err(e) => {
				logger.push(
					vec![
						(String::from("event"), String::from("setup")),
						(String::from("module"), String::from("settings")),
						(String::from("level"), String::from("WARNING")),
					],
					Some(&format!("can not parse settings file : {}", e)),
				);

				logger.push(
					vec![
						(String::from("event"), String::from("setup")),
						(String::from("module"), String::from("settings")),
						(String::from("level"), String::from("WARNING")),
					],
					Some("falling back to default settings"),
				);

				Settings::default()
			}
		},
		Err(e) => {
			logger.push(
				vec![
					(String::from("event"), String::from("setup")),
					(String::from("module"), String::from("settings")),
					(String::from("level"), String::from("WARNING")),
				],
				Some(&format!("can not read settings file : {}", e)),
			);

			let result = Settings::default();

			if e.kind() == std::io::ErrorKind::NotFound {
				if let Some(parent) = settings_path.parent() {
					if let Err(e) = std::fs::create_dir_all(parent) {
						logger.push(
							vec![
								(String::from("event"), String::from("setup")),
								(String::from("module"), String::from("settings")),
								(String::from("level"), String::from("WARNING")),
							],
							Some(&format!(
								"can not creating parent folders of settings file : {}",
								e
							)),
						);
					}
				}

				match std::fs::write(settings_path, toml::to_vec(&result).unwrap()) {
					Ok(_) => {
						logger.push(
							vec![
								(String::from("event"), String::from("setup")),
								(String::from("module"), String::from("settings")),
								(String::from("level"), String::from("INFO")),
							],
							Some("creating default settings file"),
						);
					}
					Err(e) => {
						logger.push(
							vec![
								(String::from("event"), String::from("setup")),
								(String::from("module"), String::from("settings")),
								(String::from("level"), String::from("WARNING")),
							],
							Some(&format!("can not creating default settings file : {}", e)),
						);
					}
				}
			}

			logger.push(
				vec![
					(String::from("event"), String::from("setup")),
					(String::from("module"), String::from("settings")),
					(String::from("level"), String::from("WARNING")),
				],
				Some("falling back to default settings"),
			);

			result
		}
	};

	settings
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Settings {
	pub force_https: Option<bool>,
	pub domain: Option<String>,
	pub domain_suffix: Option<String>,
	pub port: usize,
	// TODO : pub admin_email: String,
	pub token_lifetime_seconds: u64,
	pub logfile_path: String,
	pub userfile_path: String,
	pub data_path: String,
	pub https: Option<SettingsHTTPS>,
}
impl Default for Settings {
	fn default() -> Self {
		Self {
			force_https: None,
			domain: Some(String::new()),
			domain_suffix: Some(String::new()),
			port: 7541,
			// admin_email: String::new(),
			token_lifetime_seconds: 60 * 60,
			logfile_path: String::from("database/logs.msgpack"),
			userfile_path: String::from("database/users.bin"),
			data_path: String::from("database/data"),
			https: Some(SettingsHTTPS::default()),
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
