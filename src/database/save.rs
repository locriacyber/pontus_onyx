#[derive(serde::Deserialize, serde::Serialize)]
pub struct DataDocument {
	pub datastruct_version: String,
	pub etag: crate::Etag,
	pub content_type: crate::ContentType,
	pub last_modified: chrono::DateTime<chrono::Utc>,
}
impl Default for DataDocument {
	fn default() -> Self {
		Self {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: crate::Etag::new(),
			content_type: crate::ContentType::from("application/octet-stream"),
			last_modified: chrono::Utc::now(),
		}
	}
}
impl std::convert::TryFrom<crate::Item> for DataDocument {
	type Error = String;

	fn try_from(input: crate::Item) -> Result<Self, Self::Error> {
		match input {
			crate::Item::Document {
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

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DataFolder {
	pub datastruct_version: String,
	pub etag: crate::Etag,
}
impl Default for DataFolder {
	fn default() -> Self {
		Self {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: crate::Etag::new(),
		}
	}
}
