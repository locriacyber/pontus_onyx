#![allow(clippy::needless_return)]

mod client;

#[derive(Debug, Clone, serde::Serialize)]
pub enum Item {
	Folder {
		content: std::collections::HashMap<String, Box<Item>>,
	},
	Document {
		content: Vec<u8>,
	},
}

pub mod database {
	pub struct Database {
		content: std::collections::HashMap<String, Box<crate::Item>>,
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
					content: b"TODO".to_vec(),
				}),
			);
			content_c.insert(
				String::from("e"),
				Box::new(crate::Item::Folder {
					content: std::collections::HashMap::new(),
				}),
			);
			content_b.insert(
				String::from("c"),
				Box::new(crate::Item::Folder { content: content_c }),
			);
			content_a.insert(
				String::from("b"),
				Box::new(crate::Item::Folder { content: content_b }),
			);
			content.insert(
				String::from("a"),
				Box::new(crate::Item::Folder { content: content_a }),
			);

			return Ok(Self { content });
		}
		pub fn from_path(_path: &std::path::Path) -> Result<Self, CreateError> {
			todo!()
		}
	}

	/*
	TODO :
		As a special exceptions, GET and HEAD requests to a document (but
		not a folder) whose path starts with '/public/' are always allowed.
		They, as well as OPTIONS requests, can be made without a bearer
		token.
	*/
	impl Database {
		fn fetch_item(&self, request: &[&str]) -> Result<Option<&Box<crate::Item>>, FetchError> {
			// TODO : what if request == &[""] or &[] ?
			let mut result = self
				.content
				.get(&String::from(*request.first().unwrap()));

			for &request_name in request.iter().skip(1).filter(|&&e| e != "") {
				if let Some(item) = result {
					match &**item {
						crate::Item::Folder {
							content: folder_content,
						} => {
							result = folder_content.get(request_name);
						}
						crate::Item::Document { content: _ } => {
							return Err(FetchError::FolderDocumentConflict);
						}
					}
				}
			}

			return Ok(result);
		}
		fn fetch_item_mut(&mut self, request: &[&str]) -> Result<Option<&mut Box<crate::Item>>, FetchError> {
			// TODO : what if request == &[""] or &[] ?
			let mut result = self
				.content
				.get_mut(&String::from(*request.first().unwrap()));

			for &request_name in request.iter().skip(1).filter(|&&e| e != "") {
				if let Some(item) = result {
					match &mut **item {
						crate::Item::Folder {
							content: folder_content,
						} => {
							result = folder_content.get_mut(request_name);
						}
						crate::Item::Document { content: _ } => {
							return Err(FetchError::FolderDocumentConflict);
						}
					}
				}
			}

			return Ok(result);
		}
	}

	#[derive(Debug)]
	enum FetchError{
		FolderDocumentConflict,
	}

	impl Database {
		pub fn read(&self, request: &[&str]) -> Result<Option<crate::Item>, ReadError> {
			// TODO : If a document with document_name <x> exists, then no folder with folder_name <x> can exist in the same parent folder, and vice versa.

			return match request.iter().enumerate().fold(true, |acc, (i, &e)| {
				acc && crate::path::is_ok(e, i == (request.len() - 1))
			}) {
				true => {
					match self.fetch_item(request) {
						Ok(Some(result)) => Ok(Some((**result).clone())),
						Ok(None) => Ok(None),
						Err(FetchError::FolderDocumentConflict) => Err(ReadError::FolderDocumentConflict),
					}
				}
				false => Err(ReadError::WrongPath),
			};
		}

		pub fn update(
			&mut self,
			request: &[&str],
			document_update: crate::Item,
		) -> Result<String, UpdateError> {
			if let crate::Item::Document {
				content: new_content,
			} = document_update
			{
				return match request
					.iter()
					.fold(true, |acc, &e| acc && crate::path::is_ok(e, false))
				{
					true => {
						match self.fetch_item_mut(request) {
							Ok(Some(e)) => {
								if let crate::Item::Document {
									content: old_content,
								} = &mut **e
								{
									// TODO : set/update ETag ?
									*old_content = new_content;

									Ok(String::from("TODO"))
								} else {
									Err(UpdateError::NotFound)
								}
							}
							Ok(None) => Err(UpdateError::NotFound),
							Err(FetchError::FolderDocumentConflict) => Err(UpdateError::FolderDocumentConflict),
						}
					}
					false => Err(UpdateError::WrongPath),
				};
			} else {
				Err(UpdateError::DoesNotWorksForFolders)
			}
		}

		pub fn delete(
			&mut self,
			request: &[&str],
		) -> Result<String, DeleteError> {
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

			return match request.iter().enumerate().fold(true, |acc, (i, &e)| {
				acc && crate::path::is_ok(e, i == (request.len() - 1))
			}) {
				true => {
					todo!()
				},
				false => Err(DeleteError::WrongPath),
			};
		}
	}

	#[derive(Debug)]
	pub enum CreateError {}

	#[derive(Debug)]
	pub enum ReadError {
		WrongPath,
		FolderDocumentConflict,
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
			_ => !path.contains("\0"),
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
