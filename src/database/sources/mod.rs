#[cfg(feature = "server_file_storage")]
pub mod folder;
#[cfg(feature = "server_local_storage")]
pub mod local_storage;
pub mod memory;

#[cfg(feature = "server_file_storage")]
pub use folder::FolderStorage;
#[cfg(feature = "server_local_storage")]
pub use local_storage::LocalStorage;
pub use memory::MemoryStorage;

// TODO : File{file_path: std::path::PathBuf},

/// Specify how the database should interact with its data.
pub trait DataSource: std::fmt::Debug + Send {
	fn get(
		&self,
		path: &crate::ItemPath,
		if_match: &crate::Etag,
		if_none_match: &[&crate::Etag],
		get_content: bool,
	) -> Result<crate::Item, Box<dyn std::error::Error>>;

	fn put(
		&mut self,
		path: &crate::ItemPath,
		if_match: &crate::Etag,
		if_none_match: &[&crate::Etag],
		new_item: crate::Item,
	) -> crate::database::PutResult;

	fn delete(
		&mut self,
		path: &crate::ItemPath,
		if_match: &crate::Etag,
	) -> Result<crate::Etag, Box<dyn std::error::Error>>;
}
