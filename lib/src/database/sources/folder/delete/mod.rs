mod error;
pub use error::*;

#[cfg(test)]
pub mod tests;

pub fn delete(
	root_folder_path: &std::path::Path,
	path: &crate::item::ItemPath,
	if_match: &crate::item::Etag,
) -> Result<crate::item::Etag, Box<dyn std::error::Error>> {
	if path.is_folder() {
		return Err(Box::new(DeleteError::DoesNotWorksForFolders));
	}

	match super::get::get(root_folder_path, path, if_match, &[], false) {
		Ok(target_item) => {
			let old_target_item = target_item.get_etag().clone();

			let target_file_path = root_folder_path
				.join(&format!("{}", path.parent().unwrap()))
				.join(format!(".{}.itemdata.toml", path.file_name()));
			if target_file_path.exists() {
				if let Err(error) = std::fs::remove_file(&target_file_path) {
					return Err(Box::new(DeleteError::CanNotDelete {
						os_path: target_file_path,
						error: format!("{}", error),
					}));
				}
			}

			match std::fs::remove_file(root_folder_path.join(std::path::PathBuf::from(path))) {
				Ok(()) => {
					for parent in path
						.ancestors()
						.into_iter()
						.take(path.ancestors().len().saturating_sub(1))
						.rev()
					{
						if std::fs::read_dir(
							&root_folder_path.join(std::path::PathBuf::from(&parent)),
						)
						.unwrap()
						.filter(|e| e.as_ref().unwrap().file_name() != ".folder.itemdata.toml")
						.count() == 0
						{
							if root_folder_path
								.join(std::path::PathBuf::from(&parent))
								.join(".folder.itemdata.toml")
								.exists()
							{
								if let Err(error) = std::fs::remove_file(
									root_folder_path
										.join(std::path::PathBuf::from(&parent))
										.join(".folder.itemdata.toml"),
								) {
									return Err(Box::new(DeleteError::CanNotDelete {
										os_path: root_folder_path
											.join(std::path::PathBuf::from(&parent))
											.join(".folder.itemdata.toml"),
										error: format!("{}", error),
									}));
								}
							}

							if root_folder_path
								.join(std::path::PathBuf::from(&parent))
								.exists()
							{
								if let Err(error) = std::fs::remove_dir(
									root_folder_path.join(std::path::PathBuf::from(&parent)),
								) {
									return Err(Box::new(DeleteError::CanNotDelete {
										os_path: root_folder_path
											.join(std::path::PathBuf::from(&parent)),
										error: format!("{}", error),
									}));
								}
							}
						} else {
							let mut folderdata = match std::fs::read(
								root_folder_path
									.join(std::path::PathBuf::from(&parent))
									.join(".folder.itemdata.toml"),
							) {
								Ok(folderdata_content) => {
									match toml::from_slice::<crate::item::DataFolder>(
										&folderdata_content,
									) {
										Ok(res) => res,
										Err(error) => {
											return Err(Box::new(
												DeleteError::CanNotDeserializeFile {
													os_path: root_folder_path
														.join(std::path::PathBuf::from(&parent)),
													error: format!("{}", error),
												},
											));
										}
									}
								}
								Err(error) => {
									return Err(Box::new(DeleteError::CanNotReadFile {
										os_path: root_folder_path
											.join(std::path::PathBuf::from(&parent))
											.join(".folder.itemdata.toml"),
										error: format!("{}", error),
									}));
								}
							};

							folderdata.datastruct_version = String::from(env!("CARGO_PKG_VERSION"));
							folderdata.etag = crate::item::Etag::new();

							match toml::to_vec(&folderdata) {
								Ok(folderdata_content) => {
									if let Err(error) = std::fs::write(
										root_folder_path
											.join(std::path::PathBuf::from(&parent))
											.join(".folder.itemdata.toml"),
										&folderdata_content,
									) {
										return Err(Box::new(DeleteError::CanNotWriteFile {
											os_path: root_folder_path
												.join(std::path::PathBuf::from(&parent))
												.join(".folder.itemdata.toml"),
											error: format!("{}", error),
										}));
									}
								}
								Err(error) => {
									return Err(Box::new(DeleteError::CanNotSerializeFile {
										os_path: root_folder_path
											.join(std::path::PathBuf::from(&parent))
											.join(".folder.itemdata.toml"),
										error: format!("{}", error),
									}));
								}
							}
						}
					}

					return Ok(old_target_item);
				}
				Err(error) => {
					return Err(Box::new(DeleteError::CanNotDelete {
						os_path: root_folder_path.join(std::path::PathBuf::from(path)),
						error: format!("{}", error),
					}));
				}
			}
		}
		Err(boxed_error) => {
			return Err(Box::new(DeleteError::GetError(
				*boxed_error.downcast::<super::GetError>().unwrap(),
			)));
		}
	}
}
