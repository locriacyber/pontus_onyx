mod error;
pub use error::*;

#[cfg(test)]
pub mod tests;

pub fn delete(
	storage: &dyn super::Storage,
	prefix: &str,
	path: &crate::item::ItemPath,
	if_match: &crate::item::Etag,
) -> Result<crate::item::Etag, Box<dyn std::error::Error>> {
	if path.is_folder() {
		return Err(Box::new(DeleteError::DoesNotWorksForFolders));
	}

	match super::get(storage, prefix, path, if_match, &[], false) {
		Ok(crate::item::Item::Document { etag: old_etag, .. }) => {
			let parent = path
				.parent()
				.unwrap_or_else(|| crate::item::ItemPath::from(""));

			let file_path = parent
				.joined_doc(&format!(".{}.itemdata.json", path.file_name()))
				.unwrap();
			if storage
				.remove_item(&format!("{}/{}", prefix, file_path))
				.is_err()
			{
				return Err(Box::new(DeleteError::CanNotWriteFile {
					item_path: file_path,
					error: String::new(),
				}));
			}

			if storage
				.remove_item(&format!("{}/{}", prefix, file_path))
				.is_err()
			{
				return Err(Box::new(DeleteError::CanNotWriteFile {
					item_path: file_path,
					error: String::new(),
				}));
			}

			if storage
				.remove_item(&format!("{}/{}", prefix, &path))
				.is_err()
			{
				return Err(Box::new(DeleteError::CanNotWriteFile {
					item_path: path.clone(),
					error: String::new(),
				}));
			}

			for parent in path
				.ancestors()
				.into_iter()
				.take(path.ancestors().len().saturating_sub(1))
			{
				let mut has_childs = false;
				for i in 0..storage.length().unwrap_or_default() {
					let key = storage.key(i).unwrap_or_default().unwrap_or_default();
					if key.starts_with(&format!("{}/{}", prefix, parent))
						&& !key.ends_with(".itemdata.json")
					{
						has_childs = true;
						break;
					}
				}
				let mut parent_as_str = format!("{}", parent);
				parent_as_str =
					String::from(parent_as_str.strip_suffix('/').unwrap_or(&parent_as_str));
				if !parent_as_str.is_empty() {
					parent_as_str = format!("{}/", parent_as_str);
				}
				let folder_itemdata_path = crate::item::ItemPath::from(
					format!("{}/{}.folder.itemdata.json", prefix, parent_as_str).as_str(),
				);

				if !has_childs {
					if storage
						.remove_item(&format!("{}", folder_itemdata_path))
						.is_err()
					{
						return Err(Box::new(DeleteError::CanNotWriteFile {
							item_path: parent,
							error: String::new(),
						}));
					}

				// TODO : eventually cleanup folder content ? (eventually remaining *.itemdata.* files, but should not)
				} else {
					let folder_itemdata_content =
						storage.get_item(&format!("{}", folder_itemdata_path));

					match folder_itemdata_content {
						Ok(Some(folder_itemdata_content)) => {
							match serde_json::from_str::<crate::item::DataFolder>(
								&folder_itemdata_content,
							) {
								Ok(folder_itemdata) => {
									let mut new_folder_itemdata = folder_itemdata.clone();
									new_folder_itemdata.datastruct_version =
										String::from(env!("CARGO_PKG_VERSION"));
									new_folder_itemdata.etag = crate::item::Etag::new();

									match serde_json::to_string(&new_folder_itemdata) {
										Ok(new_folder_itemdata_content) => {
											if storage
												.set_item(
													&format!("{}", folder_itemdata_path),
													&new_folder_itemdata_content,
												)
												.is_err()
											{
												return Err(Box::new(
													DeleteError::CanNotWriteFile {
														item_path: folder_itemdata_path,
														error: String::new(),
													},
												));
											}
										}
										Err(error) => {
											return Err(Box::new(
												DeleteError::CanNotSerializeFile {
													item_path: folder_itemdata_path,
													error: format!("{}", error),
												},
											));
										}
									}
								}
								Err(error) => {
									return Err(Box::new(DeleteError::CanNotDeserializeFile {
										item_path: folder_itemdata_path,
										error: format!("{}", error),
									}));
								}
							}
						}
						Ok(None) => {
							match serde_json::to_string(&crate::item::DataFolder::default()) {
								Ok(new_folder_itemdata_content) => {
									if storage
										.set_item(
											&format!("{}", folder_itemdata_path),
											&new_folder_itemdata_content,
										)
										.is_err()
									{
										return Err(Box::new(DeleteError::CanNotWriteFile {
											item_path: folder_itemdata_path,
											error: String::new(),
										}));
									}
								}
								Err(error) => {
									return Err(Box::new(DeleteError::CanNotSerializeFile {
										item_path: folder_itemdata_path,
										error: format!("{}", error),
									}));
								}
							}
						}
						Err(_) => {
							return Err(Box::new(DeleteError::GetError(
								super::GetError::CanNotGetStorage,
							)));
						}
					}
				}
			}

			Ok(old_etag)
		}
		Ok(crate::item::Item::Folder { .. }) => Err(Box::new(DeleteError::DoesNotWorksForFolders)),
		Err(error) => Err(Box::new(DeleteError::GetError(*error.downcast().unwrap()))),
	}
}
