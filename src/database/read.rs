impl super::Database {
	pub fn read(&self, path: &str) -> Result<Option<crate::Item>, ReadError> {
		// TODO : If a document with document_name <x> exists, then no folder with folder_name <x> can exist in the same parent folder, and vice versa.
		let paths: Vec<&str> = path.split('/').collect();

		if paths
			.iter()
			.enumerate()
			.all(|(i, &e)| super::path::is_ok(e, i == (paths.len() - 1)))
		{
			match self.fetch_item(&paths) {
				Ok(Some(result)) => Ok(Some(result.clone())),
				Ok(None) => Ok(None),
				Err(super::FetchError::FolderDocumentConflict) => {
					Err(ReadError::FolderDocumentConflict)
				}
			}
		} else {
			Err(ReadError::WrongPath)
		}
	}
}

#[derive(Debug)]
pub enum ReadError {
	WrongPath,
	FolderDocumentConflict,
}

// TODO : return hints ?
#[cfg(feature = "server")]
impl std::convert::From<ReadError> for actix_web::Error {
	fn from(error: ReadError) -> Self {
		match error {
			ReadError::WrongPath => actix_web::HttpResponse::BadRequest()
				.content_type("application/ld+json")
				.body(r#"{"http_code":400,"http_description":"bad request"}"#)
				.into(),
			ReadError::FolderDocumentConflict => actix_web::HttpResponse::Conflict()
				.content_type("application/ld+json")
				.body(r#"{"http_code":409,"http_description":"conflict"}"#)
				.into(),
		}
	}
}
