impl super::Database {
	pub fn update(
		&mut self,
		path: &str,
		document_update: crate::Item,
	) -> Result<String, UpdateError> {
		let paths: Vec<&str> = path.split('/').collect();

		match document_update {
			crate::Item::Document {
				etag: new_etag,
				content: new_content,
				content_type: new_content_type,
				last_modified: _,
			} => {
				if paths.iter().all(|e| super::path::is_ok(e, false)) {
					match self.fetch_item_mut(&paths) {
						Ok(Some(e)) => {
							if let crate::Item::Document {
								etag: old_etag,
								content: old_content,
								content_type: old_content_type,
								last_modified: old_last_modified,
							} = e
							{
								if *old_content != new_content {
									*old_etag = new_etag.clone();
									*old_content = new_content;
									*old_content_type = new_content_type;
									*old_last_modified = chrono::Utc::now();

									// TODO : check if not modified

									match Self::update_folders_etags(
										&mut self.content,
										&mut paths.iter().cloned().take(paths.len()),
									) {
										Ok(()) => Ok(new_etag),
										Err(e) => Err(UpdateError::UpdateFoldersEtagsError(e)),
									}
								} else {
									Err(UpdateError::NotModified)
								}
							} else {
								Err(UpdateError::FolderDocumentConflict)
							}
						}
						Ok(None) => Err(UpdateError::NotFound),
						Err(super::FetchError::FolderDocumentConflict) => {
							Err(UpdateError::FolderDocumentConflict)
						}
					}
				} else {
					Err(UpdateError::WrongPath)
				}
			}
			crate::Item::Folder {
				etag: _,
				content: _,
			} => Err(UpdateError::DoesNotWorksForFolders),
		}
	}
}

#[derive(Debug)]
pub enum UpdateError {
	WrongPath,
	FolderDocumentConflict,
	DoesNotWorksForFolders,
	NotFound,
	InternalError,
	NotModified,
	UpdateFoldersEtagsError(crate::database::UpdateFoldersEtagsError),
}
