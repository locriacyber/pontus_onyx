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
		get_content: bool,
	) -> Result<crate::Item, Box<dyn std::error::Error>> {
		match self {
			Self::Memory { root_item } => memory::get(&root_item, path, if_match, if_none_match),
			Self::Folder { root_folder_path } => folder::get(
				&root_folder_path,
				path,
				if_match,
				if_none_match,
				get_content,
			),
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
	) -> Result<crate::Etag, Box<dyn std::error::Error>> {
		match self {
			Self::Memory { root_item } => memory::delete(root_item, path, if_match),
			Self::Folder { root_folder_path } => folder::delete(root_folder_path, path, if_match),
		}
	}
}
