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
			let should_be_document = paths.last().unwrap() != &"";

			if should_be_document {
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
						Some(old_version) => Ok(old_version.get_etag()),
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
}
