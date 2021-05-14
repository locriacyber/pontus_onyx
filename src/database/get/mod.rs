impl super::Database {
	pub fn get(
		&self,
		path: &str,
		if_match: Option<&str>,
		if_none_match: Option<Vec<&str>>,
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
									if let Some(none_match) = if_none_match {
										let none_match: Vec<String> = none_match
											.iter()
											.map(|s| s.trim().replace('"', ""))
											.filter(|e| !String::is_empty(e))
											.collect();

										if none_match.iter().any(|s| s == folder_etag || s == "*") {
											return Err(ErrorGet::IfNoneMatch);
										}
									}

									if let Some(if_match) = if_match {
										let if_match = if_match.trim().replace('"', "");

										if !if_match.is_empty() && folder_etag != &if_match {
											return Err(ErrorGet::IfMatchNotFound);
										}
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
								if let Some(none_match) = if_none_match {
									let none_match: Vec<String> = none_match
										.iter()
										.map(|s| s.trim().replace('"', ""))
										.filter(|e| !String::is_empty(e))
										.collect();

									if none_match.iter().any(|s| s == document_etag || s == "*") {
										return Err(ErrorGet::IfNoneMatch);
									}
								}

								if let Some(if_match) = if_match {
									let if_match = if_match.trim().replace('"', "");

									if !if_match.is_empty() && document_etag != &if_match {
										return Err(ErrorGet::IfMatchNotFound);
									}
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
