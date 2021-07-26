#[cfg(feature = "server_file_storage")]
pub mod folder;
#[cfg(feature = "server_local_storage")]
pub mod local_storage;
pub mod memory;

/// Specify where the database should save its data.
///
/// See `server_local_storage` and `server_file_storage` features to add or disable other variants.
///
/// See `database::sources` (private) modules to find their implementation.
#[non_exhaustive]
#[derive(Debug)]
pub enum DataSource {
	#[cfg(feature = "server_local_storage")]
	/// Store data in web browser's localStorage.
	///
	/// It is a local key-value database available in modern web browsers.
	///
	/// [More on MDN](https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage)
	LocalStorage {
		/// The prefix of the keys inside the localStorage.
		prefix: String,
	},
	/// Store data only in R.A.M.
	///
	/// Warning, all data disapears when database is dropped from memory !
	///
	/// This storage is useful in context without other storage or ephemeral systems,
	/// like sandboxes without filesystem or unit tests, for example.
	Memory {
		/// All data is stored inside `content` of this item, so it should be only the [`Folder`][`crate::Item::Folder`] variant.
		root_item: crate::Item,
	},
	#[cfg(feature = "server_file_storage")]
	/// Store data inside a folder from the file system.
	///
	/// Data should be saved as several nested files and folders.
	///
	/// Metadata (like ETag for example) are stored inside `*.itemdata.*` files,
	/// which are serialization of [`DataFolder`][`crate::DataFolder`] and [`DataDocument`][`crate::DataDocument`].
	Folder {
		/// The path of the folder inside the file system where to store data.
		root_folder_path: std::path::PathBuf,
	},
	// TODO : File{file_path: std::path::PathBuf},
}

impl DataSource {
	pub fn get(
		&self,
		path: &std::path::Path,
		if_match: &crate::Etag,
		if_none_match: &[&crate::Etag],
		get_content: bool,
	) -> Result<crate::Item, Box<dyn std::error::Error>> {
		match self {
			Self::Memory { root_item } => memory::get(&root_item, path, if_match, if_none_match),
			#[cfg(feature = "server_file_storage")]
			Self::Folder { root_folder_path } => folder::get(
				&root_folder_path,
				path,
				if_match,
				if_none_match,
				get_content,
			),
			#[cfg(feature = "server_local_storage")]
			Self::LocalStorage { prefix } => match web_sys::window() {
				Some(window) => match window.local_storage() {
					Ok(Some(local_storage)) => local_storage::get(
						&local_storage,
						prefix,
						path,
						if_match,
						if_none_match,
						get_content,
					),
					Ok(None) => Err(Box::new(
						super::local_storage::LocalStorageError::ThereIsNoLocalStorage,
					)),
					Err(_) => Err(Box::new(
						super::local_storage::LocalStorageError::CanNotGetLocalStorage,
					)),
				},
				None => Err(Box::new(
					super::local_storage::LocalStorageError::CanNotGetWindow,
				)),
			},
		}
	}

	pub fn put(
		&mut self,
		path: &std::path::Path,
		if_match: &crate::Etag,
		if_none_match: &[&crate::Etag],
		item: crate::Item,
	) -> crate::database::PutResult {
		match self {
			Self::Memory { root_item } => {
				memory::put(root_item, path, if_match, if_none_match, item)
			}
			#[cfg(feature = "server_file_storage")]
			Self::Folder { root_folder_path } => {
				folder::put(root_folder_path, path, if_match, if_none_match, item)
			}
			#[cfg(feature = "server_local_storage")]
			Self::LocalStorage { prefix } => match web_sys::window() {
				Some(window) => match window.local_storage() {
					Ok(Some(local_storage)) => local_storage::put(
						&local_storage,
						prefix,
						path,
						if_match,
						if_none_match,
						item,
					),
					Ok(None) => crate::database::PutResult::Err(Box::new(
						super::local_storage::LocalStorageError::ThereIsNoLocalStorage,
					)),
					Err(_) => crate::database::PutResult::Err(Box::new(
						super::local_storage::LocalStorageError::CanNotGetLocalStorage,
					)),
				},
				None => crate::database::PutResult::Err(Box::new(
					super::local_storage::LocalStorageError::CanNotGetWindow,
				)),
			},
		}
	}

	pub fn delete(
		&mut self,
		path: &std::path::Path,
		if_match: &crate::Etag,
	) -> Result<crate::Etag, Box<dyn std::error::Error>> {
		match self {
			Self::Memory { root_item } => memory::delete(root_item, path, if_match),
			#[cfg(feature = "server_file_storage")]
			Self::Folder { root_folder_path } => folder::delete(root_folder_path, path, if_match),
			#[cfg(feature = "server_local_storage")]
			Self::LocalStorage { prefix } => match web_sys::window() {
				Some(window) => match window.local_storage() {
					Ok(Some(local_storage)) => {
						local_storage::delete(&local_storage, prefix, path, if_match)
					}
					Ok(None) => Err(Box::new(
						super::local_storage::LocalStorageError::ThereIsNoLocalStorage,
					)),
					Err(_) => Err(Box::new(
						super::local_storage::LocalStorageError::CanNotGetLocalStorage,
					)),
				},
				None => Err(Box::new(
					super::local_storage::LocalStorageError::CanNotGetWindow,
				)),
			},
		}
	}
}
