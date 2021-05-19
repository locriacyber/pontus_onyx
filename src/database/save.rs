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
	pub fn save_event_into(
		&self,
		event: super::Event,
		source: super::DataSource,
	) -> Result<(), SaveError> {
		match source {
			super::DataSource::Memory(_) => {
				Ok(()) // noting to do
			}
			super::DataSource::File(file_path) => {
				if !file_path.exists() {
					match file_path.extension() {
						Some(_) => {
							if let Some(parent) = file_path.parent() {
								if let Err(e) = std::fs::create_dir_all(parent) {
									return Err(SaveError::CanNotCreateParentDirs(
										parent.to_path_buf(),
										e,
									));
								}
							}

							if let Err(e) = std::fs::write(&file_path, b"") {
								return Err(SaveError::CanNotWriteFile(file_path, e));
							}
						}
						None => {
							if let Err(e) = std::fs::create_dir_all(&file_path) {
								return Err(SaveError::CanNotCreateParentDirs(
									file_path.clone(),
									e,
								));
							}
						}
					}
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
							let filename = split_path.clone().last();
							if filename.is_none() {
								return Err(SaveError::WrongFileName(item_path));
							}
							let filename = filename.unwrap();

							let mut folder_path = data_folder;
							for part in split_path.clone().take(split_path.count() - 1) {
								folder_path.push(part);
							}
							// TODO : create .folder.podata.toml for this folders
							if let Err(e) = std::fs::create_dir_all(folder_path.clone()) {
								return Err(SaveError::CanNotCreateParentDirs(folder_path, e));
							}

							let mut document_path = folder_path.clone();
							document_path.push(filename);
							if let Err(e) = std::fs::write(&document_path, document_content) {
								return Err(SaveError::CanNotWriteFile(document_path, e));
							}

							let file_content = toml::to_string(&DataDocument {
								datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
								etag: document_etag,
								content_type: document_content_type,
								last_modified: document_last_modified,
							});
							if let Err(e) = file_content {
								return Err(SaveError::CanNotSerializeData(e));
							}
							let file_content = file_content.unwrap();

							let mut podata_path = folder_path;
							podata_path.push(format!(".{}.podata.toml", filename));
							if let Err(e) = std::fs::write(&podata_path, file_content) {
								return Err(SaveError::CanNotWriteFile(podata_path, e));
							}
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
							let filename = split_path.clone().last();
							if filename.is_none() {
								return Err(SaveError::WrongFileName(item_path));
							}
							let filename = filename.unwrap();

							let mut folder_path = data_folder;
							for part in split_path.clone().take(split_path.count() - 1) {
								folder_path.push(part);
							}
							if let Err(e) = std::fs::create_dir_all(&folder_path) {
								return Err(SaveError::CanNotCreateParentDirs(folder_path, e));
							}

							let mut document_path = folder_path.clone();
							document_path.push(filename);
							if let Err(e) = std::fs::write(document_path, document_content) {
								return Err(SaveError::CanNotWriteFile(folder_path, e));
							}

							let file_content = toml::to_string(&DataDocument {
								datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
								etag: document_etag,
								content_type: document_content_type,
								last_modified: document_last_modified,
							});
							if let Err(e) = file_content {
								return Err(SaveError::CanNotSerializeData(e));
							}
							let file_content = file_content.unwrap();

							let mut podata_path = folder_path;
							podata_path.push(format!(".{}.podata.toml", filename));
							if let Err(e) = std::fs::write(&podata_path, file_content) {
								return Err(SaveError::CanNotWriteFile(podata_path, e));
							}
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

					Ok(())
				} else if file_path.is_file() {
					let datasave = bincode::serialize(&DataMonolyth {
						datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
						content: self.content.clone(),
					});
					if let Err(e) = datasave {
						return Err(SaveError::CanNotSerializeMonolyth(e));
					}
					let datasave = datasave.unwrap();

					match std::fs::write(&file_path, datasave) {
						Ok(_) => Ok(()),
						Err(e) => Err(SaveError::CanNotWriteFile(file_path, e)),
					}
				} else {
					Err(SaveError::WrongFileType(file_path))
				}
			}
		}
	}
}

#[cfg(feature = "server_bin")]
pub enum SaveError {
	CanNotWriteFile(std::path::PathBuf, std::io::Error),
	CanNotCreateParentDirs(std::path::PathBuf, std::io::Error),
	CanNotSerializeData(toml::ser::Error),
	CanNotSerializeMonolyth(bincode::Error),
	WrongFileType(std::path::PathBuf),
	WrongFileName(String),
}
impl std::fmt::Display for SaveError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::CanNotWriteFile(path, e) => f.write_fmt(format_args!(
				"can not write file `{}` : {}",
				path.to_string_lossy(),
				e
			)),
			Self::CanNotCreateParentDirs(path, e) => f.write_fmt(format_args!(
				"can not create parent directories of `{}` : {}",
				path.to_string_lossy(),
				e
			)),
			Self::CanNotSerializeData(e) => {
				f.write_fmt(format_args!("can not serialize data : {}", e))
			}
			Self::CanNotSerializeMonolyth(e) => {
				f.write_fmt(format_args!("can not serialize monolyth : {}", e))
			}
			Self::WrongFileType(path) => f.write_fmt(format_args!(
				"`{}` is a wrong file type",
				path.to_string_lossy()
			)),
			Self::WrongFileName(path) => f.write_fmt(format_args!(
				"file name can not be extracted from path `{}`",
				path
			)),
		}
	}
}
