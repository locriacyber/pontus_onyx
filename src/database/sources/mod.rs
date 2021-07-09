pub mod folder;
pub mod memory;

#[derive(Debug)]
pub enum DataSource {
	// TODO : LocalStorage{prefix: String},
	Memory {
		root_item: crate::Item,
	},
	// TODO : File{file_path: std::path::PathBuf},
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
		_recursive: bool,
	) -> Result<crate::Item, Box<dyn std::any::Any>> {
		match self {
			Self::Memory { root_item } => {
				match memory::get(&root_item, path, if_match, if_none_match) {
					Ok(item) => Ok(item), // TODO : item.empty_clone() if recursive = false
					Err(e) => Err(e),
				}
			}
			Self::Folder {
				root_folder_path: _,
			} => {
				// TODO : path.starts_with("public/")

				/*
				TODO :
				match folder::get(&root_folder_path, path, recursive) {
					Ok(item) => Ok(item),
					Err(e) => Err(Box::new(e)),
				}
				*/
				Ok(crate::Item::new_folder(vec![]))
			}
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
			Self::Folder { root_folder_path } => {
				folder::put(root_folder_path, path, if_match, if_none_match, item)
			}
		}
	}

	pub fn delete(
		&mut self,
		path: &std::path::Path,
		if_match: &crate::Etag,
	) -> Result<crate::Etag, Box<dyn std::any::Any>> {
		match self {
			Self::Memory { root_item } => memory::delete(root_item, path, if_match),
			Self::Folder { root_folder_path } => folder::delete(root_folder_path, path, if_match),
		}
	}
}
