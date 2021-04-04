impl super::Database {
	pub fn create(
		&mut self,
		path: &str,
		new_content: &[u8],
		content_type: &str,
	) -> Result<String, CreateError> {
		let paths: Vec<&str> = path.split('/').collect();

		if paths.iter().all(|e| super::path::is_ok(e, false)) {
			if let crate::Item::Folder {
				etag: _,
				content: root_folder_content,
			} = &mut self.content
			{
				match Self::build_folders(
					root_folder_content,
					&mut paths.iter().cloned().take(paths.len() - 1),
				) {
					Ok(()) => {
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

										// TODO : check content of paths.last()
										folder_content.insert(
											String::from(*paths.last().unwrap()),
											Box::new(crate::Item::Document {
												etag: etag.clone(),
												content: new_content.to_vec(),
												content_type: String::from(content_type),
												last_modified: chrono::Utc::now(),
											}),
										);

										match Self::update_folders_etags(
											&mut self.content,
											&mut paths.iter().cloned().take(paths.len() - 1),
										) {
											Ok(()) => Ok(etag),
											Err(e) => Err(CreateError::UpdateFoldersEtagsError(e)),
										}
									}
									_ => todo!(),
								}
							}
							Err(super::FetchError::FolderDocumentConflict) => {
								Err(CreateError::FolderDocumentConflict)
							}
						}
					}
					Err(e) => Err(CreateError::FolderBuildError(e)),
				}
			} else {
				Err(CreateError::InternalError)
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
	InternalError,
	ShouldBeFolder,
	FolderBuildError(crate::database::FolderBuildError),
	UpdateFoldersEtagsError(crate::database::UpdateFoldersEtagsError),
}
