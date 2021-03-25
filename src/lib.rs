#![allow(clippy::needless_return)]

mod client;

#[derive(Debug, Clone, serde::Serialize)]
pub enum Item {
	Folder {
		etag: String,
		content: std::collections::HashMap<String, Box<Item>>,
	},
	Document {
		etag: String,
		content: Vec<u8>,
	},
}
impl Item {
	fn get_etag(&self) -> String {
		return match self {
			Self::Folder { etag, content: _ } => etag.clone(),
			Self::Document { etag, content: _ } => etag.clone(),
		};
	}
}

pub mod database {
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
		pub fn from_path(_path: &std::path::Path) -> Result<Self, CreateError> {
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
		fn fetch_item_mut(
			&mut self,
			request: &[&str],
		) -> Result<Option<&mut crate::Item>, FetchError> {
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
	}

	#[derive(Debug)]
	enum FetchError {
		FolderDocumentConflict,
	}

	impl Database {
		pub fn create(&mut self, path: &str, new_content: &[u8]) -> Result<String, CreateError> {
			let paths: Vec<&str> = path.split('/').collect();

			if paths.iter().all(|e| crate::path::is_ok(e, false)) {
				match self.fetch_item_mut(&paths) {
					Ok(Some(_e)) => Err(CreateError::AlreadyExists),
					Ok(None) => {
						let folder_path: Vec<&str> =
							paths.iter().take(paths.len() - 1).cloned().collect();

						match self.fetch_item_mut(&folder_path) {
							Ok(Some(crate::Item::Folder {
								etag: _,
								content: folder_content,
							})) => {
								let etag = ulid::Ulid::new().to_string();

								// TODO : build parent folders if not exists
								// TODO : update etag of parent folders
								// TODO : check content of paths.last()
								folder_content.insert(
									String::from(*paths.last().unwrap()),
									Box::new(crate::Item::Document {
										etag: etag.clone(),
										content: new_content.to_vec(),
									}),
								);

								Ok(etag)
							}
							_ => todo!(),
						}
					}
					Err(FetchError::FolderDocumentConflict) => {
						Err(CreateError::FolderDocumentConflict)
					}
				}
			} else {
				Err(CreateError::WrongPath)
			}
		}

		pub fn read(&self, path: &str) -> Result<Option<crate::Item>, ReadError> {
			// TODO : If a document with document_name <x> exists, then no folder with folder_name <x> can exist in the same parent folder, and vice versa.
			let paths: Vec<&str> = path.split('/').collect();

			if paths
				.iter()
				.enumerate()
				.all(|(i, &e)| crate::path::is_ok(e, i == (paths.len() - 1)))
			{
				match self.fetch_item(&paths) {
					Ok(Some(result)) => Ok(Some(result.clone())),
					Ok(None) => Ok(None),
					Err(FetchError::FolderDocumentConflict) => {
						Err(ReadError::FolderDocumentConflict)
					}
				}
			} else {
				Err(ReadError::WrongPath)
			}
		}

		pub fn update(
			&mut self,
			path: &str,
			document_update: crate::Item,
		) -> Result<String, UpdateError> {
			let paths: Vec<&str> = path.split('/').collect();

			match document_update {
				crate::Item::Document {
					etag: new_etag,
					content: new_content,
				} => {
					if paths.iter().all(|e| crate::path::is_ok(e, false)) {
						match self.fetch_item_mut(&paths) {
							Ok(Some(e)) => {
								if let crate::Item::Document {
									etag: old_etag,
									content: old_content,
								} = e
								{
									*old_etag = new_etag.clone();
									*old_content = new_content;

									Ok(new_etag)
								} else {
									Err(UpdateError::NotFound)
								}
							}
							Ok(None) => Err(UpdateError::NotFound),
							Err(FetchError::FolderDocumentConflict) => {
								Err(UpdateError::FolderDocumentConflict)
							}
						}
					} else {
						Err(UpdateError::WrongPath)
					}
				}
				crate::Item::Folder {
					etag: _,
					content: _,
				} => Err(UpdateError::DoesNotWorksForFolders),
			}
		}

		pub fn delete(&mut self, path: &str) -> Result<String, DeleteError> {
			/*
			TODO : option to keep old documents ?
				A provider MAY offer version rollback functionality to its users,
				but this specification does not define the interface for that.
			*/
			// TODO : restrain to documents only ?
			/*
			TODO:
				* the deletion of that document from the storage, and from its
					parent folder,
				* silent deletion of the parent folder if it is left empty by
					this, and so on for further ancestor folders,
				* the version of its parent folder being updated, as well as that
					of further ancestor folders.
			*/
			let paths: Vec<&str> = path.split('/').collect();

			if paths
				.iter()
				.enumerate()
				.all(|(i, &e)| crate::path::is_ok(e, i == (paths.len() - 1)))
			{
				let should_be_document = paths.last().unwrap() != &"";

				if should_be_document {
					let parent = self.fetch_item_mut(
						&paths
							.clone()
							.iter()
							.take(paths.len() - 1)
							.cloned()
							.collect::<Vec<&str>>(),
					);

					if let Ok(Some(crate::Item::Folder { etag: _, content })) = parent {
						match content.remove(*paths.last().unwrap()) {
							Some(old_version) => Ok(old_version.get_etag()),
							None => Err(DeleteError::NotFound),
						}
					} else {
						Err(DeleteError::NotFound)
					}
				} else {
					Err(DeleteError::DoesNotWorksForFolders)
				}
			} else {
				Err(DeleteError::WrongPath)
			}
		}
	}

	#[derive(Debug)]
	pub enum CreateError {
		AlreadyExists,
		WrongPath,
		FolderDocumentConflict,
		DoesNotWorksForFolders,
		NotFound,
	}

	#[derive(Debug)]
	pub enum ReadError {
		WrongPath,
		FolderDocumentConflict,
	}

	#[cfg(feature = "server")]
	impl std::convert::From<ReadError> for actix_web::Error {
		fn from(_error: ReadError) -> Self {
			todo!()
		}
	}

	#[derive(Debug)]
	pub enum UpdateError {
		WrongPath,
		FolderDocumentConflict,
		DoesNotWorksForFolders,
		NotFound,
	}

	#[derive(Debug)]
	pub enum DeleteError {
		WrongPath,
		FolderDocumentConflict,
		DoesNotWorksForFolders,
		NotFound,
	}
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

/*
TODO : Bearer tokens and access control
	* <module> string SHOULD be lower-case alphanumerical, other
		than the reserved word 'public'
	* <level> can be ':r' or ':rw'.

	<module> ':rw') any requests to paths relative to <storage_root>
					that start with '/' <module> '/' or
					'/public/' <module> '/',
	<module> ':r') any GET or HEAD requests to paths relative to
					<storage_root> that start with
					'/' <module> '/' or '/public/' <module> '/',
*/
