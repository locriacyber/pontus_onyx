#![allow(clippy::needless_return)]

mod client;
mod database;

pub use database::*;

#[derive(Debug, Clone, serde::Serialize)]
pub enum Item {
	Folder {
		etag: String,
		content: std::collections::HashMap<String, Box<Item>>,
	},
	Document {
		etag: String,
		content: Vec<u8>,
		content_type: String,
		last_modified: chrono::DateTime<chrono::offset::Utc>,
	},
}
impl Item {
	pub fn new_folder(easy_content: Vec<(&str, Self)>) -> Self {
		let mut content = std::collections::HashMap::new();
		for (name, item) in easy_content {
			content.insert(String::from(name), Box::new(item));
		}

		return Self::Folder {
			etag: ulid::Ulid::new().to_string(),
			content,
		};
	}
}
impl Item {
	fn get_etag(&self) -> String {
		return match self {
			Self::Folder { etag, content: _ } => etag.clone(),
			Self::Document {
				etag,
				content: _,
				content_type: _,
				last_modified: _,
			} => etag.clone(),
		};
	}
	pub fn is_empty(&self) -> bool {
		return match self {
			Self::Document{ etag: _, content: _, content_type: _, last_modified: _ } => false,
			Self::Folder{ etag: _, content } => content.iter().all(|(_, item)| item.is_empty()),
		};
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
