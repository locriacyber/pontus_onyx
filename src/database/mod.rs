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
	pub fn from_item_folder(content: crate::Item) -> Result<Self, put::PutError> {
		match content {
			crate::Item::Folder {
				etag: _,
				content: _,
			} => Ok(Self { content }),
			crate::Item::Document {
				etag: _,
				content: _,
				content_type: _,
				last_modified: _,
			} => Err(put::PutError::WorksOnlyForDocument),
		}
	}
	pub fn from_bytes(_bytes: &[u8]) -> Result<Self, put::PutError> {
		todo!()
	}
	pub fn from_path(_path: &std::path::Path) -> Result<Self, put::PutError> {
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
