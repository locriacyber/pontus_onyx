use std::sync::{Arc, Mutex};

pub fn load_or_create_database(
	settings: &super::Settings,
	logger: Arc<Mutex<charlie_buffalo::Logger>>,
) -> Arc<Mutex<crate::database::Database>> {
	let database = Arc::new(Mutex::new(crate::database::Database::new(
		Box::new(crate::database::sources::FolderStorage {
			root_folder_path: std::path::PathBuf::from(settings.data_path.clone()),
		}),
	)));

	logger.lock().unwrap().push(
		vec![
			(String::from("event"), String::from("setup")),
			(String::from("module"), String::from("database")),
			(String::from("level"), String::from("INFO")),
		],
		Some("database loaded or created"),
	);

	return database;
}
