use std::sync::{Arc, Mutex};

pub fn load_or_create_database(settings: &super::Settings) -> Arc<Mutex<pontus_onyx::Database>> {
	let db_path = std::path::PathBuf::from(settings.data_path.clone());
	let data_source = pontus_onyx::database::DataSource::File(db_path.clone());

	let (database, change_receiver) = match pontus_onyx::Database::new(data_source.clone()) {
		Ok(e) => e,
		Err(pontus_onyx::database::ErrorNewDatabase::FileDoesNotExists) => {
			println!(
				"\t⚠ Database does not exists in {}.",
				db_path.to_str().unwrap_or_default()
			);

			println!();
			println!("\t\t✔ New empty database created.");
			println!();

			pontus_onyx::Database::new(pontus_onyx::database::DataSource::Memory(
				pontus_onyx::Item::new_folder(vec![]),
			))
			.unwrap()
		}
		Err(e) => {
			panic!("{}", e);
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
