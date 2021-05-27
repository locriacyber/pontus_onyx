use std::sync::{Arc, Mutex};

pub fn load_or_create_database(
	settings: &super::Settings,
	logger: Arc<Mutex<charlie_buffalo::Logger>>,
) -> Arc<Mutex<pontus_onyx::Database>> {
	let db_path = std::path::PathBuf::from(settings.data_path.clone());
	let data_source = pontus_onyx::database::DataSource::File(db_path.clone());

	let logger_for_listener = logger.clone();
	let data_source_for_listener = data_source.clone();
	let event_listener = std::sync::Arc::new(std::sync::Mutex::new(move |event| {
		dbg!(&event);

		match &data_source_for_listener {
			pontus_onyx::database::DataSource::Memory(_) => { /* noting to do */ }
			pontus_onyx::database::DataSource::File(file_path) => {
				if !file_path.exists() {
					match file_path.extension() {
						Some(_) => {
							if let Some(parent) = file_path.parent() {
								if let Err(e) = std::fs::create_dir_all(parent) {
									logger_for_listener.lock().unwrap().push(
										vec![
											(String::from("module"), String::from("save")),
											(String::from("event"), format!("{:?}", &event)),
											(String::from("level"), String::from("ERROR")),
										],
										Some(&format!(
											"can not create parent directories `{}` : {}",
											parent.to_string_lossy(),
											e
										)),
									);
								}
							}

							if let Err(e) = std::fs::write(&file_path, b"") {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!(
										"can write file `{}` : {}",
										file_path.to_string_lossy(),
										e
									)),
								);
							}
						}
						None => {
							if let Err(e) = std::fs::create_dir_all(&file_path) {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!(
										"can not create parent directories `{}` : {}",
										file_path.to_string_lossy(),
										e
									)),
								);
							}
						}
					}
				}

				if file_path.is_dir() {
					let data_folder = file_path;

					match &event {
						pontus_onyx::database::Event::Create {
							path: item_path,
							item:
								pontus_onyx::Item::Document {
									etag: document_etag,
									content_type: document_content_type,
									content: document_content,
									last_modified: document_last_modified,
								},
						} => {
							let split_path = item_path.split('/');
							let filename = split_path.clone().last();
							if filename.is_none() {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!("wrong file name : `{}`", item_path)),
								);
							}
							let filename = filename.unwrap();

							let mut folder_path = data_folder.clone();
							for part in split_path.clone().take(split_path.count() - 1) {
								folder_path.push(part);
							}
							// TODO : create .folder.podata.toml for this folders
							if let Err(e) = std::fs::create_dir_all(folder_path.clone()) {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!(
										"can not create parent directories `{}` : {}",
										folder_path.to_string_lossy(),
										e
									)),
								);
							}

							let mut document_path = folder_path.clone();
							document_path.push(filename);
							if let Err(e) = std::fs::write(&document_path, document_content) {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!(
										"can not write file `{}` : {}",
										document_path.to_string_lossy(),
										e
									)),
								);
							}

							let file_content =
								toml::to_string(&pontus_onyx::database::DataDocument {
									datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
									etag: document_etag.clone(),
									content_type: document_content_type.clone(),
									last_modified: *document_last_modified,
								});
							if let Err(e) = file_content {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!("can not serialize document data : {}", e)),
								);
							} else {
								let file_content = file_content.unwrap();

								let mut podata_path = folder_path;
								podata_path.push(format!(".{}.podata.toml", filename));
								if let Err(e) = std::fs::write(&podata_path, file_content) {
									logger_for_listener.lock().unwrap().push(
										vec![
											(String::from("module"), String::from("save")),
											(String::from("event"), format!("{:?}", &event)),
											(String::from("level"), String::from("ERROR")),
										],
										Some(&format!(
											"can not create write file `{}` : {}",
											podata_path.to_string_lossy(),
											e
										)),
									);
								}
							}
						}
						pontus_onyx::database::Event::Update {
							path: item_path,
							item:
								pontus_onyx::Item::Document {
									etag: document_etag,
									content_type: document_content_type,
									content: document_content,
									last_modified: document_last_modified,
								},
						} => {
							let split_path = item_path.split('/');
							let filename = split_path.clone().last();
							if filename.is_none() {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!("wrong file name : `{}`", item_path)),
								);
							}
							let filename = filename.unwrap();

							let mut folder_path = data_folder.clone();
							for part in split_path.clone().take(split_path.count() - 1) {
								folder_path.push(part);
							}
							if let Err(e) = std::fs::create_dir_all(&folder_path) {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!(
										"can not create parent directories `{}` : {}",
										folder_path.to_string_lossy(),
										e
									)),
								);
							}

							let mut document_path = folder_path.clone();
							document_path.push(filename);
							if let Err(e) = std::fs::write(document_path, document_content) {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!(
										"can not write file`{}` : {}",
										folder_path.to_string_lossy(),
										e
									)),
								);
							}

							let file_content =
								toml::to_string(&pontus_onyx::database::DataDocument {
									datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
									etag: document_etag.clone(),
									content_type: document_content_type.clone(),
									last_modified: *document_last_modified,
								});
							if let Err(e) = file_content {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!("can not serialize document data : {}", e)),
								);
							} else {
								let file_content = file_content.unwrap();

								let mut podata_path = folder_path;
								podata_path.push(format!(".{}.podata.toml", filename));
								if let Err(e) = std::fs::write(&podata_path, file_content) {
									logger_for_listener.lock().unwrap().push(
										vec![
											(String::from("module"), String::from("save")),
											(String::from("event"), format!("{:?}", &event)),
											(String::from("level"), String::from("ERROR")),
										],
										Some(&format!(
											"can not write file `{}` : {}",
											podata_path.to_string_lossy(),
											e
										)),
									);
								}
							}
						}
						pontus_onyx::database::Event::Delete { path: item_path } => {
							let split_path = item_path.split('/');
							let filename = split_path.clone().last();
							if filename.is_none() {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!("wrong file name : {}", item_path)),
								);
							}
							let filename = filename.unwrap();

							let mut folder_path = data_folder.clone();
							for part in split_path.clone().take(split_path.count() - 1) {
								folder_path.push(part);
							}

							let mut file_path = folder_path.clone();
							file_path.push(filename);

							let mut podata_path = folder_path;
							podata_path.push(format!(".{}.podata.toml", filename));

							if let Err(e) = std::fs::remove_file(&file_path) {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!(
										"can not write file `{}` : {}",
										file_path.to_string_lossy(),
										e
									)),
								);
							}

							if let Err(e) = std::fs::remove_file(&podata_path) {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!(
										"can not write file `{}` : {}",
										podata_path.to_string_lossy(),
										e
									)),
								);
							}
						}
						pontus_onyx::database::Event::Create {
							path: item_path,
							item: pontus_onyx::Item::Folder {
								etag: folder_etag, ..
							},
						} => {
							let split_path = item_path.split('/');

							let mut folder_path = data_folder.clone();
							for part in split_path.filter(|e| !e.is_empty()) {
								folder_path.push(part);
							}

							// TODO : generate .podata.toml for parent folders
							if let Err(e) = std::fs::create_dir_all(&folder_path) {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!(
										"can not create parent directories `{}` : {}",
										folder_path.to_string_lossy(),
										e
									)),
								);
							}

							let file_content =
								toml::to_string(&pontus_onyx::database::DataFolder {
									datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
									etag: folder_etag.clone(),
								});
							if let Err(e) = file_content {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!("can not serialize folder data : {}", e)),
								);
							} else {
								let file_content = file_content.unwrap();

								let mut podata_path = folder_path;
								podata_path.push(".folder.podata.toml");
								if let Err(e) = std::fs::write(&podata_path, file_content) {
									logger_for_listener.lock().unwrap().push(
										vec![
											(String::from("module"), String::from("save")),
											(String::from("event"), format!("{:?}", &event)),
											(String::from("level"), String::from("ERROR")),
										],
										Some(&format!(
											"can not write file `{}` : {}",
											podata_path.to_string_lossy(),
											e
										)),
									);
								}
							}
						}
						pontus_onyx::database::Event::Update {
							path: item_path,
							item: pontus_onyx::Item::Folder {
								etag: folder_etag, ..
							},
						} => {
							let split_path = item_path.split('/');

							let mut folder_path = data_folder.clone();
							for part in split_path.filter(|e| !e.is_empty()) {
								folder_path.push(part);
							}

							let file_content =
								toml::to_string(&pontus_onyx::database::DataFolder {
									datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
									etag: folder_etag.clone(),
								});
							if let Err(e) = file_content {
								logger_for_listener.lock().unwrap().push(
									vec![
										(String::from("module"), String::from("save")),
										(String::from("event"), format!("{:?}", &event)),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!("can not serialize folder data : {}", e)),
								);
							} else {
								let file_content = file_content.unwrap();

								let mut podata_path = folder_path;
								podata_path.push(".folder.podata.toml");
								if let Err(e) = std::fs::write(&podata_path, file_content) {
									logger_for_listener.lock().unwrap().push(
										vec![
											(String::from("module"), String::from("save")),
											(String::from("event"), format!("{:?}", &event)),
											(String::from("level"), String::from("ERROR")),
										],
										Some(&format!(
											"can not write file `{}` : {}",
											podata_path.to_string_lossy(),
											e
										)),
									);
								}
							}
						}
					}
				} else {
					logger_for_listener.lock().unwrap().push(
						vec![
							(String::from("module"), String::from("save")),
							(String::from("event"), format!("{:?}", &event)),
							(String::from("level"), String::from("ERROR")),
						],
						Some(&format!(
							"wrong file type for `{}`",
							file_path.to_string_lossy()
						)),
					);
				}
			}
		}
		// TODO : restore mononolyth save ?
	}));

	let database = Arc::new(Mutex::new(
		match pontus_onyx::Database::new(&data_source, Some(event_listener.clone())) {
			Ok(result) => {
				logger.lock().unwrap().push(
					vec![
						(String::from("event"), String::from("setup")),
						(String::from("module"), String::from("database")),
						(String::from("level"), String::from("INFO")),
					],
					Some(&format!(
						"database succesfully loaded from `{}`",
						db_path.to_string_lossy()
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
						db_path.to_string_lossy()
					)),
				);

				let res = pontus_onyx::Database::new(
					&pontus_onyx::database::DataSource::Memory(pontus_onyx::Item::new_folder(
						vec![],
					)),
					Some(event_listener),
				)
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
		},
	));

	return database;
}
