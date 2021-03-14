#![allow(clippy::needless_return)]

#[derive(Debug, Clone, serde::Serialize)]
pub enum Item {
	Folder {
		name: String,
		content: std::collections::HashMap<String, Box<Item>>,
	},
	Document {
		name: String,
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
					name: String::from("d"),
					content: b"TODO".to_vec(),
				}),
			);
			content_c.insert(
				String::from("e"),
				Box::new(crate::Item::Folder {
					name: String::from("e"),
					content: std::collections::HashMap::new(),
				}),
			);
			content_b.insert(
				String::from("c"),
				Box::new(crate::Item::Folder {
					name: String::from("c"),
					content: content_c,
				}),
			);
			content_a.insert(
				String::from("b"),
				Box::new(crate::Item::Folder {
					name: String::from("b"),
					content: content_b,
				}),
			);
			content.insert(
				String::from("a"),
				Box::new(crate::Item::Folder {
					name: String::from("a"),
					content: content_a,
				}),
			);

			return Ok(Self { content });
		}
		pub fn from_path(_path: &std::path::Path) -> Result<Self, CreateError> {
			todo!()
		}
	}

	impl Database {
		pub fn get(&self, request: &[&str]) -> Result<Option<crate::Item>, GetError> {
			// TODO : If a document with document_name <x> exists, then no folder with folder_name <x> can exist in the same parent folder, and vice versa.

			return match request.iter().enumerate().fold(true, |acc, (i, &e)| {
				acc && crate::path::is_ok(e, i == (request.len() - 1))
			}) {
				true => {
					// TODO : what if request == &[""] or &[] ?
					let mut result = self.content.get(&String::from(*request.first().unwrap()));
					for &request_name in request.iter().skip(1).filter(|&&e| e != "") {
						if let Some(item) = result {
							match &**item {
								crate::Item::Folder {
									name: _,
									content: folder_content,
								} => {
									result = folder_content.get(request_name);
								}
								crate::Item::Document {
									name: _,
									content: _,
								} => {
									return Err(GetError::FolderDocumentConflict);
								}
							}
						}
					}

					Ok(match result {
						Some(result) => Some((**result).clone()),
						None => None,
					})
				}
				false => Err(GetError::WrongPath),
			};
		}
	}

	#[derive(Debug)]
	pub enum CreateError {}

	#[derive(Debug)]
	pub enum GetError {
		WrongPath,
		FolderDocumentConflict,
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
