impl super::Database {
	pub fn get(
		&self,
		path: &str,
		if_match: &str,
		if_none_match: crate::IfNoneMatch,
	) -> Result<&crate::Item, ErrorGet> {
		let paths: Vec<&str> = path.split('/').collect();
		let should_be_folder = paths.last().unwrap().is_empty();

		if paths
			.iter()
			.enumerate()
			.all(|(i, &e)| super::utils::path::is_ok(e, i == (paths.len() - 1)))
		{
			match self.fetch_item(&paths) {
				Ok(Some(item)) => {
					match &item {
						crate::Item::Folder {
							etag: folder_etag, ..
						} => {
							if should_be_folder {
								if path.starts_with("public") {
									return Err(ErrorGet::CanNotBeListed);
								} else {
									// TODO : weak headers ?
									if if_none_match
										.into_iter()
										.any(|s| &s == folder_etag && !s.is_empty() || s == "*")
									{
										return Err(ErrorGet::IfNoneMatch);
									}

									if !if_match.is_empty() && folder_etag != if_match {
										return Err(ErrorGet::IfMatchNotFound);
									}

									return Ok(item);
								}
							} else {
								return Err(ErrorGet::Conflict);
							}
						}
						crate::Item::Document {
							etag: document_etag,
							..
						} => {
							if !should_be_folder {
								if if_none_match
									.iter()
									.any(|s| s == document_etag && !s.is_empty() || *s == "*")
								{
									return Err(ErrorGet::IfNoneMatch);
								}

								if !if_match.is_empty() && document_etag != if_match {
									return Err(ErrorGet::IfMatchNotFound);
								}

								return Ok(item);
							} else {
								return Err(ErrorGet::NotFound);
							}
						}
					}
				}
				Ok(None) => Err(ErrorGet::NotFound),
				Err(super::FetchError::FolderDocumentConflict) => Err(ErrorGet::Conflict),
			}
		} else {
			Err(ErrorGet::WrongPath)
		}
	}
}

mod error;
pub use error::*;
