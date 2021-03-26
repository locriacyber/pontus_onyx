impl super::Database {
	pub fn create(&mut self, path: &str, new_content: &[u8]) -> Result<String, CreateError> {
		let paths: Vec<&str> = path.split('/').collect();

		if paths.iter().all(|e| super::path::is_ok(e, false)) {
			match self.fetch_item_mut(&paths) {
				Ok(Some(_e)) => Err(CreateError::AlreadyExists),
				Ok(None) => {
					let folder_path: Vec<&str> =
						paths.iter().take(paths.len() - 1).cloned().collect();

					match self.fetch_item_mut(&folder_path) {
						Ok(Some(crate::Item::Folder {
							etag: _,
							content: folder_content,
						})) => {
							let etag = ulid::Ulid::new().to_string();

							// TODO : build parent folders if not exists
							// TODO : update etag of parent folders
							// TODO : check content of paths.last()
							folder_content.insert(
								String::from(*paths.last().unwrap()),
								Box::new(crate::Item::Document {
									etag: etag.clone(),
									content: new_content.to_vec(),
								}),
							);

							Ok(etag)
						}
						_ => todo!(),
					}
				}
				Err(super::FetchError::FolderDocumentConflict) => {
					Err(CreateError::FolderDocumentConflict)
				}
			}
		} else {
			Err(CreateError::WrongPath)
		}
	}
}

#[derive(Debug)]
pub enum CreateError {
	AlreadyExists,
	WrongPath,
	FolderDocumentConflict,
	DoesNotWorksForFolders,
	NotFound,
}
