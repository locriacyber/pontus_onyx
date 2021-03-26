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
			} => {
				if paths.iter().all(|e| super::path::is_ok(e, false)) {
					match self.fetch_item_mut(&paths) {
						Ok(Some(e)) => {
							if let crate::Item::Document {
								etag: old_etag,
								content: old_content,
							} = e
							{
								*old_etag = new_etag.clone();
								*old_content = new_content;

								Ok(new_etag)
							} else {
								Err(UpdateError::NotFound)
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
}
