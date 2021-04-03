impl super::Database {
	pub fn delete(&mut self, path: &str) -> Result<String, DeleteError> {
		/*
		TODO : option to keep old documents ?
			A provider MAY offer version rollback functionality to its users,
			but this specification does not define the interface for that.
		*/
		// TODO : restrain to documents only ?
		/*
		TODO:
			* the deletion of that document from the storage, and from its
				parent folder,
			* silent deletion of the parent folder if it is left empty by
				this, and so on for further ancestor folders,
			* the version of its parent folder being updated, as well as that
				of further ancestor folders.
		*/
		let paths: Vec<&str> = path.split('/').collect();

		if paths
			.iter()
			.enumerate()
			.all(|(i, &e)| super::path::is_ok(e, i == (paths.len() - 1)))
		{
			if paths.last().unwrap() != &"" {
				let parent = self.fetch_item_mut(
					&paths
						.clone()
						.iter()
						.take(paths.len() - 1)
						.cloned()
						.collect::<Vec<&str>>(),
				);

				if let Ok(Some(crate::Item::Folder { etag: _, content })) = parent {
					match content.remove(*paths.last().unwrap()) {
						Some(old_version) => {
							match Self::update_folders_etags(
								&mut self.content,
								&mut paths.iter().cloned().take(paths.len() - 1),
							) {
								Ok(()) => {
									for i in 0..=paths.len() {
										self.cleanup_empty_folders(
											&paths
												.iter()
												.take(paths.len() - i)
												.fold(String::new(), |acc, e| acc + *e + "/"),
										)
										.ok(); // errors are not important here
									}

									Ok(old_version.get_etag())
								}
								Err(e) => Err(DeleteError::UpdateFoldersEtagsError(e)),
							}
						}
						None => Err(DeleteError::NotFound),
					}
				} else {
					Err(DeleteError::NotFound)
				}
			} else {
				Err(DeleteError::DoesNotWorksForFolders)
			}
		} else {
			Err(DeleteError::WrongPath)
		}
	}
}

#[derive(Debug)]
pub enum DeleteError {
	WrongPath,
	FolderDocumentConflict,
	DoesNotWorksForFolders,
	NotFound,
	UpdateFoldersEtagsError(crate::UpdateFoldersEtagsError),
}
