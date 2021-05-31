#![allow(clippy::needless_return)]

#[cfg(feature = "client_lib")]
pub mod client;

#[cfg(feature = "server_lib")]
pub mod database;

pub type ItemPath = String;
pub type Etag = String;
pub type IfMatch = String;
pub type IfNoneMatch = Vec<String>;
pub type ContentType = String;

#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub enum Item {
	Folder {
		etag: crate::Etag,
		content: std::collections::HashMap<crate::ItemPath, Box<crate::Item>>,
	},
	Document {
		etag: crate::Etag,
		#[derivative(Debug = "ignore")]
		content: Vec<u8>,
		content_type: crate::ContentType,
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
			Self::Folder { etag, .. } => etag.clone(),
			Self::Document { etag, .. } => etag.clone(),
		};
	}
	pub fn is_empty(&self) -> bool {
		return match self {
			Self::Document { .. } => false,
			Self::Folder { content, .. } => content.iter().all(|(_, item)| item.is_empty()),
		};
	}
}
