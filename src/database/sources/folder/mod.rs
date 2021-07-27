mod delete;
mod get;
mod put;

pub use delete::DeleteError;
pub use get::GetError;
pub use put::PutError;

/// Store data inside a folder from the file system.
///
/// Data should be saved as several nested files and folders.
///
/// Metadata (like ETag for example) are stored inside `*.itemdata.*` files,
/// which are serialization of [`DataFolder`][`crate::DataFolder`] and [`DataDocument`][`crate::DataDocument`].
#[derive(Debug)]
pub struct FolderStorage {
	/// The path of the folder inside the file system where to store data.
	pub root_folder_path: std::path::PathBuf,
}
impl crate::database::DataSource for FolderStorage {
	fn get(
		&self,
		path: &std::path::Path,
		if_match: &crate::Etag,
		if_none_match: &[&crate::Etag],
		get_content: bool,
	) -> Result<crate::Item, Box<dyn std::error::Error>> {
		get::get(
			&self.root_folder_path,
			path,
			if_match,
			if_none_match,
			get_content,
		)
	}

	fn put(
		&mut self,
		path: &std::path::Path,
		if_match: &crate::Etag,
		if_none_match: &[&crate::Etag],
		new_item: crate::Item,
	) -> crate::database::PutResult {
		put::put(
			&self.root_folder_path,
			path,
			if_match,
			if_none_match,
			new_item,
		)
	}

	fn delete(
		&mut self,
		path: &std::path::Path,
		if_match: &crate::Etag,
	) -> Result<crate::Etag, Box<dyn std::error::Error>> {
		delete::delete(&self.root_folder_path, path, if_match)
	}
}
