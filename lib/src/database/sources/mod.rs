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
		path: &crate::item::ItemPath,
		if_match: &crate::item::Etag,
		if_none_match: &[&crate::item::Etag],
		get_content: bool,
	) -> Result<crate::item::Item, Box<dyn std::error::Error>>;

	fn put(
		&mut self,
		path: &crate::item::ItemPath,
		if_match: &crate::item::Etag,
		if_none_match: &[&crate::item::Etag],
		new_item: crate::item::Item,
	) -> crate::database::PutResult;

	fn delete(
		&mut self,
		path: &crate::item::ItemPath,
		if_match: &crate::item::Etag,
	) -> Result<crate::item::Etag, Box<dyn std::error::Error>>;
}
