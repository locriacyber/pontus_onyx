mod delete;
mod get;
mod put;

pub use delete::DeleteError;
pub use get::GetError;
pub use put::{PutError, PutResult};

#[derive(Debug)]
pub struct Database {
	content: crate::Item,
}

impl Database {
	pub fn new(source: Source) -> Result<Self, NewDatabaseError> {
		match source {
			Source::Memory(item) => match item {
				crate::Item::Folder { .. } => Ok(Self {
					content: item.clone(),
				}),
				crate::Item::Document { .. } => {
					Err(NewDatabaseError::WorksOnlyForDocument)
				}
			},
			Source::File(path) => {
				if !path.exists() {
					return Err(NewDatabaseError::FileDoesNotExists);
				}

				match path_to_item(path) {
					Ok((_, data)) => {
						Ok(Self { content: *data })
					}
					Err(e) => Err(e),
				}
			}
		}
	}
}

#[derive(serde::Deserialize, serde::Serialize)]
struct PontusOnyxFileData{
	datastruct_version: String,
	etag: String,
	content_type: String,
	last_modified: chrono::DateTime<chrono::Utc>,
}
impl Default for PontusOnyxFileData {
	fn default() -> Self {
		Self{
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: ulid::Ulid::new().to_string(),
			content_type: actix_web::http::header::ContentType::octet_stream().to_string(),
			last_modified: chrono::Utc::now(),
		}
	}
}

#[derive(serde::Deserialize, serde::Serialize)]
struct PontusOnyxFolderData{
	datastruct_version: String,
	etag: String,
}
impl Default for PontusOnyxFolderData {
	fn default() -> Self {
		Self{
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: ulid::Ulid::new().to_string(),
		}
	}
}

#[derive(serde::Deserialize, serde::Serialize)]
struct PontusOnyxMonolythData{
	datastruct_version: String,
	content: crate::Item,
}
impl Default for PontusOnyxMonolythData {
	fn default() -> Self {
		Self{
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			content: crate::Item::new_folder(vec![]),
		}
	}
}

fn path_to_item(path: std::path::PathBuf) -> Result<(String, Box<crate::Item>), NewDatabaseError> {
	if path.is_dir() {
		match std::fs::read_dir(path.clone()) {
			Ok(items) => {
				let content: Vec<Result<(String, Box<crate::Item>), NewDatabaseError>> = items.map(|item| {
					match item {
						Ok(entry) => {
							if entry.path().is_dir() {
								path_to_item(entry.path())
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

								let res = match std::fs::read(entry.path()) {
									Ok(bytes) => Ok((
										filename,
										Box::new(crate::Item::Document {
											content: bytes,
											content_type: podata.content_type,
											etag: podata.etag,
											last_modified: podata.last_modified,
										}),
									)),
									Err(e) => Err(NewDatabaseError::IOError(e)),
								};

								res
							} else {
								panic!("todo")
							}
						}
						Err(e) => Err(NewDatabaseError::IOError(e)),
					}
				}).filter(|e| {
					if let Ok((name, _)) = e {
						if name.ends_with(".podata.toml") {
							return false;
						}
					}

					return true;
				}).collect();

				if content.iter().all(|e| !e.is_err()) {
					let folder_name = path.file_name().unwrap().to_str().unwrap();

					let mut podata = path.clone();
					podata.push(".folder.podata.toml");

					let podata = std::fs::read(podata);

					let podata: PontusOnyxFolderData = match podata {
						Ok(podata) => toml::from_slice(&podata).unwrap_or_default(),
						Err(_) => PontusOnyxFolderData::default(),
					};

					Ok((
						String::from(folder_name),
						Box::new(crate::Item::Folder {
							etag: podata.etag,
							content: content.into_iter().map(|e| e.unwrap()).collect::<std::collections::HashMap<String, Box<crate::Item>>>(),
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
			Err(e) => Err(NewDatabaseError::IOError(e)),
		}
	} else if path.is_file() {
		match std::fs::read(&path) {
			Ok(content) => {
				let content: Result<PontusOnyxMonolythData, bincode::Error> = bincode::deserialize(&content);

				match content {
					Ok(monolyth) => Ok((String::from(path.file_name().unwrap().to_str().unwrap()), Box::new(monolyth.content))),
					Err(e) => Err(NewDatabaseError::DeserializeError(e)),
				}
			},
			Err(e) => Err(NewDatabaseError::IOError(e)),
		}
	} else {
		Err(NewDatabaseError::WrongSource)
	}
}

#[derive(Debug)]
pub enum NewDatabaseError {
	DeserializeError(bincode::Error),
	FileDoesNotExists,
	IOError(std::io::Error),
	WorksOnlyForDocument,
	WrongSource,
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

#[derive(Debug)]
pub enum FetchError {
	FolderDocumentConflict,
}

#[derive(Debug)]
pub enum FolderBuildError {
	FolderDocumentConflict,
	WrongFolderName,
}

#[derive(Debug)]
pub enum UpdateFoldersEtagsError {
	FolderDocumentConflict,
	WrongFolderName,
	MissingFolder,
}

pub enum CleanupFolderError {
	NotAFolder,
}

#[derive(Debug)]
pub enum Source {
	Memory(crate::Item),
	File(std::path::PathBuf),
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
