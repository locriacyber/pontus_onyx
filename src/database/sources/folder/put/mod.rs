mod error;
pub use error::*;

#[cfg(test)]
pub mod tests;

pub fn put(
	root_folder_path: &std::path::Path,
	path: &crate::item::ItemPath,
	if_match: &crate::item::Etag,
	if_none_match: &[&crate::item::Etag],
	new_item: crate::item::Item,
) -> crate::database::PutResult {
	// TODO : test if path is document and new_item is folder (and vice-versa) ?
	if path.is_folder() {
		return crate::database::PutResult::Err(Box::new(PutError::DoesNotWorksForFolders));
	}

	let item_fetch = super::get::get(root_folder_path, path, if_match, if_none_match, true);

	let target_content_path = root_folder_path.join(std::path::PathBuf::from(path));
	let target_data_path = root_folder_path
		.join(std::path::PathBuf::from(
			&path
				.parent()
				.unwrap_or_else(|| crate::item::ItemPath::from("")),
		))
		.join(format!(".{}.itemdata.toml", path.file_name()));

	match item_fetch {
		Ok(crate::item::Item::Document {
			content: old_content,
			content_type: old_content_type,
			..
		}) => {
			if let crate::item::Item::Document {
				content: new_content,
				content_type: new_content_type,
				last_modified: new_last_modified,
				..
			} = new_item
			{
				if new_content != old_content || new_content_type != old_content_type {
					let new_etag = crate::item::Etag::new();

					for parent_path in path
						.ancestors()
						.into_iter()
						.take(path.ancestors().len().saturating_sub(1))
					{
						let target_parent_path =
							root_folder_path.join(std::path::PathBuf::from(&parent_path));
						let parent_datafile_path = target_parent_path.join(".folder.itemdata.toml");

						let mut parent_datafile: crate::item::DataFolder = {
							let file_content = std::fs::read(&parent_datafile_path);
							match file_content {
								Ok(file_content) => match toml::from_slice(&file_content) {
									Ok(file_content) => file_content,
									Err(error) => {
										return crate::database::PutResult::Err(Box::new(
											PutError::CanNotDeserializeFile {
												os_path: parent_datafile_path,
												error: format!("{}", error),
											},
										));
									}
								},
								Err(error) => {
									return crate::database::PutResult::Err(Box::new(
										PutError::CanNotReadFile {
											os_path: parent_datafile_path,
											error: format!("{}", error),
										},
									));
								}
							}
						};

						parent_datafile.datastruct_version =
							String::from(env!("CARGO_PKG_VERSION"));
						parent_datafile.etag = crate::item::Etag::new();

						match toml::to_vec(&parent_datafile) {
							Ok(parent_datafile) => {
								if let Err(error) =
									std::fs::write(&parent_datafile_path, &parent_datafile)
								{
									return crate::database::PutResult::Err(Box::new(
										PutError::CanNotWriteFile {
											os_path: parent_datafile_path,
											error: format!("{}", error),
										},
									));
								}
							}
							Err(error) => {
								return crate::database::PutResult::Err(Box::new(
									PutError::CanNotSerializeFile {
										os_path: parent_datafile_path,
										error: format!("{}", error),
									},
								));
							}
						}
					}

					if let Some(new_content) = new_content {
						if let Err(error) = std::fs::write(&target_content_path, &new_content) {
							return crate::database::PutResult::Err(Box::new(
								PutError::CanNotWriteFile {
									os_path: target_content_path,
									error: format!("{}", error),
								},
							));
						}
					}

					match toml::to_vec(&crate::item::DataDocument {
						datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
						etag: new_etag.clone(),
						content_type: new_content_type,
						last_modified: Some(time::OffsetDateTime::now_utc()),
					}) {
						Ok(datadoc) => {
							if let Err(error) = std::fs::write(&target_data_path, &datadoc) {
								return crate::database::PutResult::Err(Box::new(
									PutError::CanNotWriteFile {
										os_path: target_data_path,
										error: format!("{}", error),
									},
								));
							}
						}
						Err(error) => {
							return crate::database::PutResult::Err(Box::new(
								PutError::CanNotSerializeFile {
									os_path: target_data_path,
									error: format!("{}", error),
								},
							));
						}
					}

					return crate::database::PutResult::Updated(
						new_etag,
						new_last_modified.unwrap_or_else(time::OffsetDateTime::now_utc),
					);
				} else {
					return crate::database::PutResult::Err(Box::new(PutError::ContentNotChanged));
				}
			} else {
				return crate::database::PutResult::Err(Box::new(PutError::DoesNotWorksForFolders));
			}
		}
		Ok(crate::item::Item::Folder { .. }) => {
			return crate::database::PutResult::Err(Box::new(super::GetError::Conflict {
				item_path: path.clone(),
			}));
		}
		Err(boxed_error) => {
			let get_error = *boxed_error.downcast::<super::GetError>().unwrap();

			if let super::GetError::NotFound { item_path: _ } = get_error {
				if let crate::item::Item::Document {
					content: new_content,
					content_type: new_content_type,
					last_modified: new_last_modified,
					..
				} = new_item
				{
					let new_etag = crate::item::Etag::new();

					for parent_path in path
						.ancestors()
						.into_iter()
						.take(path.ancestors().len().saturating_sub(1))
					{
						let target_parent_path =
							root_folder_path.join(std::path::PathBuf::from(&parent_path));
						let parent_datafile_path = target_parent_path.join(".folder.itemdata.toml");

						if let Err(error) = std::fs::create_dir_all(&target_parent_path) {
							return crate::database::PutResult::Err(Box::new(
								PutError::CanNotWriteFile {
									os_path: target_parent_path,
									error: format!("{}", error),
								},
							));
						}

						let parent_datafile = crate::item::DataFolder {
							datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
							etag: crate::item::Etag::new(),
						};

						match toml::to_vec(&parent_datafile) {
							Ok(datafile) => {
								if let Err(error) = std::fs::write(&parent_datafile_path, &datafile)
								{
									return crate::database::PutResult::Err(Box::new(
										PutError::CanNotWriteFile {
											os_path: parent_datafile_path,
											error: format!("{}", error),
										},
									));
								}
							}
							Err(error) => {
								return crate::database::PutResult::Err(Box::new(
									PutError::CanNotSerializeFile {
										os_path: parent_datafile_path,
										error: format!("{}", error),
									},
								));
							}
						}
					}

					if let Some(new_content) = new_content {
						if let Err(error) = std::fs::write(&target_content_path, &new_content) {
							return crate::database::PutResult::Err(Box::new(
								PutError::CanNotWriteFile {
									os_path: target_content_path,
									error: format!("{}", error),
								},
							));
						}
					}

					match toml::to_vec(&crate::item::DataDocument {
						datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
						etag: new_etag.clone(),
						content_type: new_content_type,
						last_modified: Some(time::OffsetDateTime::now_utc()),
					}) {
						Ok(datafile) => {
							if let Err(error) = std::fs::write(&target_data_path, &datafile) {
								return crate::database::PutResult::Err(Box::new(
									PutError::CanNotWriteFile {
										os_path: target_data_path,
										error: format!("{}", error),
									},
								));
							}
						}
						Err(error) => {
							return crate::database::PutResult::Err(Box::new(
								PutError::CanNotSerializeFile {
									os_path: target_data_path,
									error: format!("{}", error),
								},
							));
						}
					}

					return crate::database::PutResult::Updated(
						new_etag,
						new_last_modified.unwrap_or_else(time::OffsetDateTime::now_utc),
					);
				} else {
					return crate::database::PutResult::Err(Box::new(
						PutError::DoesNotWorksForFolders,
					));
				}
			} else {
				return crate::database::PutResult::Err(Box::new(PutError::GetError(get_error)));
			}
		}
	}
}
