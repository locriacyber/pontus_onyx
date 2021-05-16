#[derive(serde::Deserialize, serde::Serialize)]
pub struct DataDocument {
	pub datastruct_version: String,
	pub etag: String,
	pub content_type: String,
	pub last_modified: chrono::DateTime<chrono::Utc>,
}
impl Default for DataDocument {
	fn default() -> Self {
		Self {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: ulid::Ulid::new().to_string(),
			content_type: String::from("application/octet-stream"),
			last_modified: chrono::Utc::now(),
		}
	}
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DataFolder {
	pub datastruct_version: String,
	pub etag: String,
}
impl Default for DataFolder {
	fn default() -> Self {
		Self {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: ulid::Ulid::new().to_string(),
		}
	}
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DataMonolyth {
	pub datastruct_version: String,
	pub content: crate::Item,
}
impl Default for DataMonolyth {
	fn default() -> Self {
		Self {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			content: crate::Item::new_folder(vec![]),
		}
	}
}

#[cfg(feature = "server_bin")]
pub fn do_not_handle_events(handle: std::sync::mpsc::Receiver<crate::database::Event>) {
	std::thread::spawn(move || loop {
		handle.recv().ok();
	});
}

#[cfg(feature = "server_bin")]
impl super::Database {
	pub fn save_event_into(&self, event: super::Event, source: super::DataSource) {
		match source {
			super::DataSource::Memory(_) => {} // noting to do
			super::DataSource::File(file_path) => {
				if !file_path.exists() {
					match file_path.extension() {
						Some(_) => {
							if let Some(parent) = file_path.parent() {
								std::fs::create_dir_all(parent);
							}
							std::fs::write(&file_path, b"")
						}
						None => std::fs::create_dir_all(&file_path),
					}
					.unwrap();
				}

				if file_path.is_dir() {
					let data_folder = file_path;

					match event {
						super::Event::Create {
							path: item_path,
							item:
								crate::Item::Document {
									etag: document_etag,
									content_type: document_content_type,
									content: document_content,
									last_modified: document_last_modified,
								},
						} => {
							let split_path = item_path.split('/');
							let filename = split_path.clone().last().unwrap();

							let mut folder_path = data_folder;
							for part in split_path.clone().take(split_path.count() - 1) {
								folder_path.push(part);
							}
							// TODO : create .folder.podata.toml for this folders
							std::fs::create_dir_all(folder_path.clone()).unwrap();

							let mut document_path = folder_path.clone();
							document_path.push(filename);
							std::fs::write(document_path, document_content);

							let mut podata_path = folder_path;
							podata_path.push(format!(".{}.podata.toml", filename));
							std::fs::write(
								podata_path,
								toml::to_string(&DataDocument {
									datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
									etag: document_etag,
									content_type: document_content_type,
									last_modified: document_last_modified,
								})
								.unwrap(),
							);
						}
						super::Event::Update {
							path: item_path,
							item:
								crate::Item::Document {
									etag: document_etag,
									content_type: document_content_type,
									content: document_content,
									last_modified: document_last_modified,
								},
						} => {
							let split_path = item_path.split('/');
							let filename = split_path.clone().last().unwrap();

							let mut folder_path = data_folder;
							for part in split_path.clone().take(split_path.count() - 1) {
								folder_path.push(part);
							}
							std::fs::create_dir_all(folder_path.clone()).unwrap();

							let mut document_path = folder_path.clone();
							document_path.push(filename);
							std::fs::write(document_path, document_content);

							let mut podata_path = folder_path;
							podata_path.push(format!(".{}.podata.toml", filename));
							std::fs::write(
								podata_path,
								toml::to_string(&DataDocument {
									datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
									etag: document_etag,
									content_type: document_content_type,
									last_modified: document_last_modified,
								})
								.unwrap(),
							);
						}
						super::Event::Delete { .. } => todo!(),
						super::Event::Create {
							item: crate::Item::Folder { .. },
							..
						} => todo!(),
						super::Event::Update {
							item: crate::Item::Folder { .. },
							..
						} => todo!(),
					}
				} else if file_path.is_file() {
					let datasave = bincode::serialize(&DataMonolyth {
						datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
						content: self.content.clone(),
					})
					.unwrap();

					std::fs::write(file_path, datasave);
				}
			}
		}
	}
}
