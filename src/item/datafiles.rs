/// Used to (de)serialize metadata of [`Document`][`crate::item::Item::Document`] in/from a file.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct DataDocument {
	pub datastruct_version: String,
	pub etag: crate::item::Etag,
	pub content_type: crate::item::ContentType,
	pub last_modified: Option<time::OffsetDateTime>,
}
impl Default for DataDocument {
	fn default() -> Self {
		Self {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: crate::item::Etag::new(),
			content_type: crate::item::ContentType::from("application/octet-stream"),
			last_modified: Some(time::OffsetDateTime::now_utc()),
		}
	}
}
impl std::convert::TryFrom<crate::item::Item> for DataDocument {
	type Error = String;

	fn try_from(input: crate::item::Item) -> Result<Self, Self::Error> {
		match input {
			crate::item::Item::Document {
				etag,
				content_type,
				last_modified,
				..
			} => Ok(Self {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag,
				content_type,
				last_modified,
			}),
			_ => Err(String::from("input should be Item::Document and it is not")),
		}
	}
}

/// Used to (de)serialize metadata of [`Folder`][`crate::item::Item::Folder`] in/from a file.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct DataFolder {
	pub datastruct_version: String,
	pub etag: crate::item::Etag,
}
impl Default for DataFolder {
	fn default() -> Self {
		Self {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: crate::item::Etag::new(),
		}
	}
}
