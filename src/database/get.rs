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
			.all(|(i, &e)| super::path::is_ok(e, i == (paths.len() - 1)))
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

#[derive(Debug)]
pub enum ErrorGet {
	CanNotBeListed,
	Conflict,
	IfMatchNotFound,
	IfNoneMatch,
	NotFound,
	WrongPath,
}
impl std::fmt::Display for ErrorGet {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		match self {
			Self::CanNotBeListed => f.write_str("the content of this folder can not be listed"),
			Self::Conflict => f.write_str(
				"there is a conflict of name between folder and document name on the request path",
			),
			Self::IfMatchNotFound => f.write_str(
				"the requested ETag was not found (specified in If-Match header of your request)",
			),
			Self::IfNoneMatch => f.write_str(
				"the unwanted ETag was found (specified in If-None-Match header of your request)",
			),
			Self::NotFound => f.write_str("requested item was not found"),
			Self::WrongPath => f.write_str("the path of the item is incorrect"),
		}
	}
}
impl std::error::Error for ErrorGet {}

#[cfg(feature = "server_bin")]
impl std::convert::From<ErrorGet> for actix_web::HttpResponse {
	fn from(input: ErrorGet) -> Self {
		let request_method = actix_web::http::Method::GET;
		match input {
			ErrorGet::CanNotBeListed => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorGet::Conflict => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::CONFLICT,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorGet::IfMatchNotFound => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorGet::IfNoneMatch => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorGet::NotFound => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorGet::WrongPath => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", input)),
				true,
			),
		}
	}
}
