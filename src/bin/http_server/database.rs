use std::sync::{Arc, Mutex};

pub fn load_or_create_database(
	settings: &super::Settings,
	logger: Arc<Mutex<charlie_buffalo::Logger>>,
) -> Arc<Mutex<pontus_onyx::Database>> {
	let db_path = std::path::PathBuf::from(settings.data_path.clone());
	let data_source = pontus_onyx::database::DataSource::File(db_path.clone());

	let (database, change_receiver) = match pontus_onyx::Database::new(data_source) {
		Ok(result) => {
			logger.lock().unwrap().push(
				vec![
					(String::from("event"), String::from("setup")),
					(String::from("module"), String::from("database")),
					(String::from("level"), String::from("INFO")),
				],
				Some(&format!(
					"database succesfully loaded from `{}`",
					db_path.to_str().unwrap_or_default()
				)),
			);

			result
		}
		Err(pontus_onyx::database::ErrorNewDatabase::FileDoesNotExists) => {
			logger.lock().unwrap().push(
				vec![
					(String::from("event"), String::from("setup")),
					(String::from("module"), String::from("database")),
					(String::from("level"), String::from("WARNING")),
				],
				Some(&format!(
					"database does not exists in `{}`",
					db_path.to_str().unwrap_or_default()
				)),
			);

			let res = pontus_onyx::Database::new(pontus_onyx::database::DataSource::Memory(
				pontus_onyx::Item::new_folder(vec![]),
			))
			.unwrap();

			logger.lock().unwrap().push(
				vec![
					(String::from("event"), String::from("setup")),
					(String::from("module"), String::from("database")),
					(String::from("level"), String::from("INFO")),
				],
				Some("new empty database created"),
			);

			res
		}
		Err(e) => {
			logger.lock().unwrap().push(
				vec![
					(String::from("event"), String::from("setup")),
					(String::from("module"), String::from("database")),
					(String::from("level"), String::from("ERROR")),
				],
				Some(&format!("{}", e)),
			);

			panic!();
		}
	};
	let database = Arc::new(Mutex::new(database));

	let database_for_save = database.clone();
	std::thread::spawn(move || loop {
		match change_receiver.recv() {
			Ok(event) => database_for_save.lock().unwrap().save_event_into(
				event,
				pontus_onyx::database::DataSource::File(db_path.clone()),
			),
			Err(e) => panic!("{}", e),
		}
	});

	return database;
}
