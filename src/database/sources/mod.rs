mod folder;
mod memory;

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
	pub fn read(
		&self,
		path: &std::path::PathBuf,
		if_match: &crate::Etag,
		if_none_match: &[&crate::Etag],
		recursive: bool,
	) -> Result<crate::Item, Box<dyn std::error::Error>> {
		match self {
			Self::Memory { root_item } => {
				match memory::read(&root_item, path, if_match, if_none_match) {
					Ok(item) => Ok(item), // TODO : item.empty_clone() if recursive = false
					Err(e) => Err(Box::new(e)),
				}
			}
			Self::Folder { root_folder_path } => {
				// TODO : path.starts_with("public/")

				/*
				TODO :
				match folder::read(&root_folder_path, path, recursive) {
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
		path: &std::path::PathBuf,
		if_match: &crate::Etag,
		if_none_match: &[&crate::Etag],
		item: crate::Item,
	) -> Result<crate::Etag, Box<dyn std::error::Error>> {
		match self {
			Self::Memory { root_item } => {
				match memory::put(root_item, path, if_match, if_none_match, item) {
					Ok(etag) => Ok(etag),
					Err(e) => Err(Box::new(e)),
				}
			}
			Self::Folder { root_folder_path } => {
				match folder::put(root_folder_path, path, if_match, if_none_match, item) {
					Ok(etag) => Ok(etag),
					Err(e) => Err(Box::new(e)),
				}
			}
		}
	}

	pub fn delete(
		&mut self,
		path: &std::path::PathBuf,
		if_match: &crate::Etag,
	) -> Result<crate::Etag, Box<dyn std::error::Error>> {
		match self {
			Self::Memory { root_item } => match memory::delete(root_item, path, if_match) {
				Ok(etag) => Ok(etag),
				Err(e) => Err(Box::new(e)),
			},
			Self::Folder { root_folder_path } => {
				match folder::delete(root_folder_path, path, if_match) {
					Ok(etag) => Ok(etag),
					Err(e) => Err(Box::new(e)),
				}
			}
		}
	}
}
