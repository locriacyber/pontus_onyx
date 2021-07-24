#[cfg(feature = "server_file_storage")]
pub mod folder;
#[cfg(feature = "server_local_storage")]
pub mod local_storage;
pub mod memory;

#[derive(Debug)]
pub enum DataSource {
	#[cfg(feature = "server_local_storage")]
	LocalStorage {
		prefix: String,
	},
	Memory {
		root_item: crate::Item,
	},
	// TODO : File{file_path: std::path::PathBuf},
	#[cfg(feature = "server_file_storage")]
	Folder {
		root_folder_path: std::path::PathBuf,
	},
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
