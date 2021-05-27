#[derive(serde::Deserialize, serde::Serialize)]
pub struct DataDocument {
	pub datastruct_version: String,
	pub etag: String,
	pub content_type: String,
	pub last_modified: chrono::DateTime<chrono::Utc>,
}
impl Default for DataDocument {
	fn default() -> Self {
		Self {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: ulid::Ulid::new().to_string(),
			content_type: String::from("application/octet-stream"),
			last_modified: chrono::Utc::now(),
		}
	}
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DataFolder {
	pub datastruct_version: String,
	pub etag: String,
}
impl Default for DataFolder {
	fn default() -> Self {
		Self {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: ulid::Ulid::new().to_string(),
		}
	}
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DataMonolyth {
	pub datastruct_version: String,
	pub content: crate::Item,
}
impl Default for DataMonolyth {
	fn default() -> Self {
		Self {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			content: crate::Item::new_folder(vec![]),
		}
	}
}
