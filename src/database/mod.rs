mod create;
mod delete;
mod read;
mod update;

pub use create::CreateError;
pub use delete::DeleteError;
pub use read::ReadError;
pub use update::UpdateError;

#[derive(Debug)]
pub struct Database {
	content: crate::Item,
}

impl Database {
	pub fn from_bytes(_bytes: &[u8]) -> Result<Self, CreateError> {
		// TODO : cleanup
		let mut content: std::collections::HashMap<String, Box<crate::Item>> =
			std::collections::HashMap::new();
		let mut content_a: std::collections::HashMap<String, Box<crate::Item>> =
			std::collections::HashMap::new();
		let mut content_b: std::collections::HashMap<String, Box<crate::Item>> =
			std::collections::HashMap::new();
		let mut content_c = std::collections::HashMap::new();

		content_c.insert(
			String::from("d"),
			Box::new(crate::Item::Document {
				etag: ulid::Ulid::new().to_string(),
				content: b"TODO".to_vec(),
			}),
		);
		content_c.insert(
			String::from("e"),
			Box::new(crate::Item::Folder {
				etag: ulid::Ulid::new().to_string(),
				content: std::collections::HashMap::new(),
			}),
		);
		content_b.insert(
			String::from("c"),
			Box::new(crate::Item::Folder {
				etag: ulid::Ulid::new().to_string(),
				content: content_c,
			}),
		);
		content_a.insert(
			String::from("b"),
			Box::new(crate::Item::Folder {
				etag: ulid::Ulid::new().to_string(),
				content: content_b,
			}),
		);
		content.insert(
			String::from("a"),
			Box::new(crate::Item::Folder {
				etag: ulid::Ulid::new().to_string(),
				content: content_a,
			}),
		);

		let mut content_0 = std::collections::HashMap::new();
		content_0.insert(
			String::from("1"),
			Box::new(crate::Item::Document {
				etag: ulid::Ulid::new().to_string(),
				content: b"01010101".to_vec(),
			}),
		);
		content.insert(
			String::from("0"),
			Box::new(crate::Item::Folder {
				etag: ulid::Ulid::new().to_string(),
				content: content_0,
			}),
		);

		return Ok(Self {
			content: crate::Item::Folder {
				etag: ulid::Ulid::new().to_string(),
				content,
			},
		});
	}
	pub fn from_path(_path: &std::path::Path) -> Result<Self, create::CreateError> {
		todo!()
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
						result = match folder_content.get(request_name) {
							Some(b) => Some(&**b),
							None => None,
						};
					}
					crate::Item::Document {
						etag: _,
						content: _,
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
						result = match folder_content.get_mut(request_name) {
							Some(b) => Some(&mut **b),
							None => None,
						};
					}
					crate::Item::Document {
						etag: _,
						content: _,
					} => {
						return Err(FetchError::FolderDocumentConflict);
					}
				}
			}
		}

		return Ok(result);
	}
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
		return match path.next() {
			Some(needed) => {
				if needed.trim().is_empty() {
					Err(UpdateFoldersEtagsError::WrongFolderName)
				} else {
					match folder {
						crate::Item::Folder {
							etag: folder_etag,
							content: folder_content,
						} => {
							*folder_etag = ulid::Ulid::new().to_string();
							match folder_content.get_mut(needed) {
								Some(item) => Self::update_folders_etags(&mut **item, path),
								None => Err(UpdateFoldersEtagsError::MissingFolder),
							}
						}
						crate::Item::Document {
							etag: _,
							content: _,
						} => Err(UpdateFoldersEtagsError::FolderDocumentConflict),
					}
				}
			}
			None => Ok(()),
		};
	}
}

#[derive(Debug)]
enum FetchError {
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
