mod delete;
mod get;
mod put;

pub use delete::ErrorDelete;
pub use get::ErrorGet;
pub use put::{ErrorPut, ResultPut};

#[derive(Debug)]
pub struct Database {
	content: crate::Item,
	changes_tx: std::sync::mpsc::Sender<crate::database::Event>,
}

impl Database {
	pub fn new(
		source: DataSource,
	) -> Result<(Self, std::sync::mpsc::Receiver<crate::database::Event>), ErrorNewDatabase> {
		match source {
			DataSource::Memory(item) => match item {
				crate::Item::Folder { .. } => {
					let (tx, rx) = std::sync::mpsc::channel();
					Ok((
						Self {
							content: item,
							changes_tx: tx,
						},
						rx,
					))
				}
				crate::Item::Document { .. } => Err(ErrorNewDatabase::WorksOnlyForFolder),
			},
			#[cfg(feature = "server_bin")]
			DataSource::File(path) => {
				let (tx, rx) = std::sync::mpsc::channel();
				match path_to_item(path, tx.clone()) {
					Ok((_, data)) => Ok((
						Self {
							content: *data,
							changes_tx: tx,
						},
						rx,
					)),
					Err(e) => Err(e),
				}
			}
		}
	}
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PontusOnyxFileData {
	pub datastruct_version: String,
	pub etag: String,
	pub content_type: String,
	pub last_modified: chrono::DateTime<chrono::Utc>,
}
impl Default for PontusOnyxFileData {
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
pub struct PontusOnyxFolderData {
	pub datastruct_version: String,
	pub etag: String,
}
impl Default for PontusOnyxFolderData {
	fn default() -> Self {
		Self {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: ulid::Ulid::new().to_string(),
		}
	}
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PontusOnyxMonolythData {
	pub datastruct_version: String,
	pub content: crate::Item,
}
impl Default for PontusOnyxMonolythData {
	fn default() -> Self {
		Self {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			content: crate::Item::new_folder(vec![]),
		}
	}
}

#[cfg(feature = "server_bin")]
fn path_to_item(
	path: std::path::PathBuf,
	tx: std::sync::mpsc::Sender<Event>,
) -> Result<(String, Box<crate::Item>), ErrorNewDatabase> {
	if !path.exists() {
		return Err(ErrorNewDatabase::FileDoesNotExists);
	}

	if path.is_dir() {
		match std::fs::read_dir(path.clone()) {
			Ok(items) => {
				let content: Vec<Result<(String, Box<crate::Item>), ErrorNewDatabase>> = items
					.map(|item| match item {
						Ok(entry) => {
							if entry.path().is_dir() {
								path_to_item(entry.path(), tx.clone())
							} else if entry.path().is_file() {
								let filename =
									String::from(entry.file_name().as_os_str().to_str().unwrap());

								let mut podata = path.clone();
								podata.push(format!(".{}.podata.toml", filename));

								let podata = std::fs::read(podata);

								let podata: PontusOnyxFileData = match podata {
									Ok(podata) => toml::from_slice(&podata).unwrap_or_default(),
									Err(_) => PontusOnyxFileData::default(),
								};

								match std::fs::read(entry.path()) {
									Ok(bytes) => Ok((
										filename,
										Box::new(crate::Item::Document {
											content: bytes,
											content_type: podata.content_type,
											etag: podata.etag,
											last_modified: podata.last_modified,
										}),
									)),
									Err(e) => Err(ErrorNewDatabase::IOError(e)),
								}
							} else {
								todo!()
							}
						}
						Err(e) => Err(ErrorNewDatabase::IOError(e)),
					})
					.filter(|e| {
						if let Ok((name, _)) = e {
							if name.ends_with(".podata.toml") {
								return false;
							}
						}

						return true;
					})
					.collect();

				if content.iter().all(|e| !e.is_err()) {
					let folder_name = path.file_name().unwrap().to_str().unwrap();

					let mut podata = path.clone();
					podata.push(".folder.podata.toml");

					let podata = std::fs::read(podata);

					let podata: PontusOnyxFolderData = match podata {
						Ok(podata) => toml::from_slice(&podata).unwrap_or_default(),
						Err(_e) => {
							let result = PontusOnyxFolderData::default();

							/* TODO :
							if let std::io::ErrorKind::NotFound = e.kind() {
								tx.send(Event::Create{})
							}
							*/

							result
						}
					};

					Ok((
						String::from(folder_name),
						Box::new(crate::Item::Folder {
							etag: podata.etag,
							content: content
								.into_iter()
								.map(|e| e.unwrap())
								.collect::<std::collections::HashMap<String, Box<crate::Item>>>(
							),
						}),
					))
				} else {
					Err(match content.into_iter().find(|e| e.is_err()) {
						Some(Ok(_)) => panic!("error #SGR-573"),
						Some(Err(e)) => e,
						None => panic!("error #NYH-812"),
					})
				}
			}
			Err(e) => Err(ErrorNewDatabase::IOError(e)),
		}
	} else if path.is_file() {
		match std::fs::read(&path) {
			Ok(content) => {
				let content: Result<PontusOnyxMonolythData, bincode::Error> =
					bincode::deserialize(&content);

				match content {
					Ok(monolyth) => Ok((
						String::from(path.file_name().unwrap().to_str().unwrap()),
						Box::new(monolyth.content),
					)),
					Err(e) => Err(ErrorNewDatabase::DeserializeError(e)),
				}
			}
			Err(e) => Err(ErrorNewDatabase::IOError(e)),
		}
	} else {
		Err(ErrorNewDatabase::WrongSource)
	}
}

#[derive(Debug)]
pub enum ErrorNewDatabase {
	DeserializeError(bincode::Error),
	FileDoesNotExists,
	IOError(std::io::Error),
	WorksOnlyForFolder,
	WrongSource,
}
impl std::fmt::Display for ErrorNewDatabase {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		match self {
			Self::DeserializeError(e) => {
				f.write_fmt(format_args!("the format of this database is wrong : {}", e))
			}
			Self::FileDoesNotExists => f.write_str("the specified file does not exists"),
			Self::IOError(e) => f.write_fmt(format_args!(
				"there is an error while reading database file : {}",
				e
			)),
			Self::WorksOnlyForFolder => f.write_str("this item should be only type Item::Folder"),
			Self::WrongSource => f.write_str("this database can not be created from this source"),
		}
	}
}

impl Database {
	fn fetch_item(&self, request: &[&str]) -> Result<Option<&crate::Item>, FetchError> {
		let mut result = Some(&self.content);

		for &request_name in request.iter().filter(|&&e| !e.is_empty()) {
			if let Some(item) = result {
				match item {
					crate::Item::Folder {
						etag: _,
						content: folder_content,
					} => {
						result = folder_content.get(request_name).map(|b| &**b);
					}
					crate::Item::Document {
						etag: _,
						content: _,
						content_type: _,
						last_modified: _,
					} => {
						return Err(FetchError::FolderDocumentConflict);
					}
				}
			}
		}

		return Ok(result);
	}
	fn fetch_item_mut(&mut self, request: &[&str]) -> Result<Option<&mut crate::Item>, FetchError> {
		let mut result = Some(&mut self.content);

		for &request_name in request.iter().filter(|&&e| !e.is_empty()) {
			if let Some(item) = result {
				match item {
					crate::Item::Folder {
						etag: _,
						content: folder_content,
					} => {
						result = folder_content.get_mut(request_name).map(|b| &mut **b);
					}
					crate::Item::Document {
						etag: _,
						content: _,
						content_type: _,
						last_modified: _,
					} => {
						return Err(FetchError::FolderDocumentConflict);
					}
				}
			}
		}

		return Ok(result);
	}
	fn cleanup_empty_folders(&mut self, path: &str) -> Result<(), CleanupFolderError> {
		let splitted_path: Vec<&str> = path.split('/').collect();

		match self.fetch_item_mut(&splitted_path) {
			Ok(Some(crate::Item::Folder { etag: _, content })) => {
				if content.is_empty() && splitted_path.len() > 1 {
					let temp = self.fetch_item_mut(
						&splitted_path
							.iter()
							.take(splitted_path.len() - 1 - 1)
							.cloned()
							.collect::<Vec<&str>>(),
					);

					if let Ok(Some(crate::Item::Folder {
						etag: _,
						content: parent_content,
					})) = temp
					{
						parent_content.remove(
							*splitted_path
								.iter()
								.filter(|p| !p.is_empty())
								.last()
								.unwrap(),
						);
					}
				}

				Ok(())
			}
			_ => Err(CleanupFolderError::NotAFolder),
		}
	}
}

impl Database {
	fn build_folders(
		content: &mut std::collections::HashMap<String, Box<crate::Item>>,
		path: &mut dyn std::iter::Iterator<Item = &str>,
	) -> Result<(), FolderBuildError> {
		return match path.next() {
			Some(needed) => {
				if needed.trim().is_empty() {
					Err(FolderBuildError::WrongFolderName)
				} else {
					match content.get_mut(needed) {
						Some(item) => match &mut **item {
							crate::Item::Folder {
								etag: _,
								content: folder_content,
							} => Self::build_folders(folder_content, path),
							crate::Item::Document {
								etag: _,
								content: _,
								content_type: _,
								last_modified: _,
							} => Err(FolderBuildError::FolderDocumentConflict),
						},
						None => {
							let mut child_content = std::collections::HashMap::new();

							let res = Self::build_folders(&mut child_content, path);

							content.insert(
								String::from(needed),
								Box::new(crate::Item::Folder {
									etag: ulid::Ulid::new().to_string(),
									content: child_content,
								}),
							);

							res
						}
					}
				}
			}
			None => Ok(()),
		};
	}
	fn update_folders_etags(
		folder: &mut crate::Item,
		path: &mut dyn std::iter::Iterator<Item = &str>,
	) -> Result<(), UpdateFoldersEtagsError> {
		let next = path.next();

		return match folder {
			crate::Item::Folder {
				etag: folder_etag,
				content: folder_content,
			} => {
				*folder_etag = ulid::Ulid::new().to_string();

				match next {
					Some(needed) => {
						if needed.trim().is_empty() {
							Err(UpdateFoldersEtagsError::WrongFolderName)
						} else {
							match folder_content.get_mut(needed) {
								Some(item) => Self::update_folders_etags(&mut **item, path),
								None => Err(UpdateFoldersEtagsError::MissingFolder),
							}
						}
					}
					None => Ok(()),
				}
			}
			crate::Item::Document {
				etag: _,
				content: _,
				content_type: _,
				last_modified: _,
			} => match next {
				Some(_) => Err(UpdateFoldersEtagsError::FolderDocumentConflict),
				None => Ok(()),
			},
		};
	}
}

#[cfg(feature = "server_bin")]
impl Database {
	pub fn save_event_into(&self, event: Event, source: DataSource) {
		match source {
			DataSource::Memory(_) => {} // noting to do
			DataSource::File(file_path) => {
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
						Event::Create {
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
								toml::to_string(&PontusOnyxFileData {
									datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
									etag: document_etag,
									content_type: document_content_type,
									last_modified: document_last_modified,
								})
								.unwrap(),
							);
						}
						Event::Update {
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
								toml::to_string(&PontusOnyxFileData {
									datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
									etag: document_etag,
									content_type: document_content_type,
									last_modified: document_last_modified,
								})
								.unwrap(),
							);
						}
						Event::Delete { .. } => todo!(),
						Event::Create {
							item: crate::Item::Folder { .. },
							..
						} => todo!(),
						Event::Update {
							item: crate::Item::Folder { .. },
							..
						} => todo!(),
					}
				} else if file_path.is_file() {
					let datasave = bincode::serialize(&PontusOnyxMonolythData {
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

#[derive(Debug)]
enum FetchError {
	FolderDocumentConflict,
}

#[derive(Debug)]
enum FolderBuildError {
	FolderDocumentConflict,
	WrongFolderName,
}

#[derive(Debug)]
enum UpdateFoldersEtagsError {
	FolderDocumentConflict,
	WrongFolderName,
	MissingFolder,
}

enum CleanupFolderError {
	NotAFolder,
}

#[derive(Debug, Clone)]
pub enum DataSource {
	Memory(crate::Item),
	#[cfg(feature = "server_bin")]
	File(std::path::PathBuf),
}

#[derive(Debug)]
pub enum Event {
	Create { path: String, item: crate::Item },
	Update { path: String, item: crate::Item },
	Delete { path: String },
}

mod path {
	pub fn is_ok(path: &str, is_last: bool) -> bool {
		return match path {
			"" => is_last,
			"." => false,
			".." => false,
			_ => !path.contains('\0'),
		};
	}

	#[test]
	fn pfuh8x4mntyi3ej() {
		let input = "gq7tib";
		assert_eq!(is_ok(input, true), true);
		assert_eq!(is_ok(input, false), true);
	}

	#[test]
	fn b2auwz1qizhfkrolm() {
		let input = "";
		assert_eq!(is_ok(input, true), true);
		assert_eq!(is_ok(input, false), false);
	}

	#[test]
	fn hf1atgq7tibjv22p2whyhrl() {
		let input = "gq7t\0ib";
		assert_eq!(is_ok(input, true), false);
		assert_eq!(is_ok(input, false), false);
	}
}
