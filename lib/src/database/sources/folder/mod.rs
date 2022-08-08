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
/// which are serialization of [`DataFolder`][`crate::item::DataFolder`] and [`DataDocument`][`crate::item::DataDocument`].
#[derive(Debug)]
pub struct FolderStorage {
	/// The path of the folder inside the file system where to store data.
	pub root_folder_path: std::path::PathBuf,
}
impl crate::database::DataSource for FolderStorage {
	fn get(
		&self,
		path: &crate::item::ItemPath,
		if_match: &crate::item::Etag,
		if_none_match: &[&crate::item::Etag],
		get_content: bool,
	) -> Result<crate::item::Item, Box<dyn std::error::Error>> {
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
		path: &crate::item::ItemPath,
		if_match: &crate::item::Etag,
		if_none_match: &[&crate::item::Etag],
		new_item: crate::item::Item,
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
		path: &crate::item::ItemPath,
		if_match: &crate::item::Etag,
	) -> Result<crate::item::Etag, Box<dyn std::error::Error>> {
		delete::delete(&self.root_folder_path, path, if_match)
	}
}
