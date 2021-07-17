use std::sync::{Arc, Mutex};

pub fn load_or_create_database(
	settings: &super::Settings,
	logger: Arc<Mutex<charlie_buffalo::Logger>>,
) -> Arc<Mutex<pontus_onyx::database::Database>> {
	let data_source = pontus_onyx::database::DataSource::Folder {
		root_folder_path: std::path::PathBuf::from(settings.data_path.clone()),
	};

	logger.lock().unwrap().push(
		vec![
			(String::from("event"), String::from("setup")),
			(String::from("module"), String::from("database")),
			(String::from("level"), String::from("INFO")),
		],
		Some("new empty database created"),
	);

	let database = Arc::new(Mutex::new(pontus_onyx::database::Database::new(
		data_source,
	)));

	return database;
}
