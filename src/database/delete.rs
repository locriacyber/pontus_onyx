impl super::Database {
	pub fn delete(&mut self, path: &str, if_match: Option<&str>) -> Result<String, ErrorDelete> {
		let if_match_result = if let Some(find_match) = if_match {
			let find_match = find_match.trim().replace('"', "");

			match self.get(path, if_match, None) {
				Ok(crate::Item::Document {
					etag: document_etag,
					..
				}) => Ok(document_etag == &find_match),
				Ok(crate::Item::Folder { .. }) => Err(ErrorDelete::WorksOnlyForDocument),
				Err(crate::database::ErrorGet::CanNotBeListed) => {
					Err(ErrorDelete::WorksOnlyForDocument)
				}
				Err(crate::database::ErrorGet::Conflict) => Err(ErrorDelete::Conflict),
				Err(crate::database::ErrorGet::IfMatchNotFound) => {
					Err(ErrorDelete::IfMatchNotFound)
				}
				Err(crate::database::ErrorGet::IfNoneMatch) => Err(ErrorDelete::IfNoneMatch),
				Err(crate::database::ErrorGet::NotFound) => Err(ErrorDelete::NotFound),
				Err(crate::database::ErrorGet::WrongPath) => Err(ErrorDelete::WrongPath),
			}
		} else {
			Ok(true)
		};

		match if_match_result {
			Ok(if_match_result) => {
				// TODO : looks like there is a lot of duplicates between this checks and self.get() errors

				if if_match_result {
					/*
					TODO : option to keep old documents ?
						A provider MAY offer version rollback functionality to its users,
						but this specification does not define the interface for that.
					*/
					let paths: Vec<&str> = path.split('/').collect();

					if paths
						.iter()
						.enumerate()
						.all(|(i, &e)| super::path::is_ok(e, i == (paths.len() - 1)))
					{
						if paths.last().unwrap() != &"" {
							match self.get(path, if_match, None) {
								Ok(crate::Item::Document {
									etag: _,
									content: _,
									content_type: _,
									last_modified: _,
								}) => {
									let parent = self.fetch_item_mut(
										&paths
											.clone()
											.iter()
											.take(paths.len() - 1)
											.cloned()
											.collect::<Vec<&str>>(),
									);

									if let Ok(Some(crate::Item::Folder { etag: _, content })) =
										parent
									{
										match content.remove(*paths.last().unwrap()) {
											Some(old_version) => {
												match Self::update_folders_etags(
													&mut self.content,
													&mut paths
														.iter()
														.cloned()
														.take(paths.len() - 1),
												) {
													Ok(()) => {
														for i in 0..=paths.len() {
															self.cleanup_empty_folders(
																&paths
																	.iter()
																	.take(paths.len() - i)
																	.fold(
																		String::new(),
																		|acc, e| acc + *e + "/",
																	),
															)
															.ok(); // errors are not important here
														}

														Ok(old_version.get_etag())
													}
													Err(e) => {
														// TODO : is following conversion is OK ?
														Err(match e {
															super::UpdateFoldersEtagsError::FolderDocumentConflict => ErrorDelete::Conflict,
															super::UpdateFoldersEtagsError::MissingFolder => ErrorDelete::NotFound,
															super::UpdateFoldersEtagsError::WrongFolderName => ErrorDelete::WrongPath,
														})
													}
												}
											}
											None => Err(ErrorDelete::NotFound),
										}
									} else {
										Err(ErrorDelete::NotFound)
									}
								}
								Ok(crate::Item::Folder {
									etag: _,
									content: _,
								}) => Err(ErrorDelete::WorksOnlyForDocument),
								Err(super::get::ErrorGet::CanNotBeListed) => {
									Err(ErrorDelete::WorksOnlyForDocument)
								} // TODO : is this OK ?
								Err(super::get::ErrorGet::Conflict) => Err(ErrorDelete::Conflict),
								Err(super::get::ErrorGet::IfMatchNotFound) => {
									Err(ErrorDelete::IfMatchNotFound)
								}
								Err(super::get::ErrorGet::IfNoneMatch) => {
									Err(ErrorDelete::InternalError)
								} // should never happen
								Err(super::get::ErrorGet::NotFound) => Err(ErrorDelete::NotFound),
								Err(super::get::ErrorGet::WrongPath) => Err(ErrorDelete::WrongPath),
							}
						} else {
							Err(ErrorDelete::WorksOnlyForDocument)
						}
					} else {
						Err(ErrorDelete::WrongPath)
					}
				} else {
					return Err(ErrorDelete::IfMatchNotFound);
				}
			}
			Err(e) => {
				return Err(e);
			}
		}
	}
}

#[derive(Debug)]
pub enum ErrorDelete {
	Conflict,
	IfMatchNotFound,
	IfNoneMatch,
	InternalError,
	NotFound,
	WorksOnlyForDocument,
	WrongPath,
}
impl std::fmt::Display for ErrorDelete {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		match self {
			Self::Conflict => f.write_str(
				"there is a conflict of name between folder and document name on the request path",
			),
			Self::IfMatchNotFound => f.write_str(
				"the requested ETag was not found (specified in If-Match header of your request)",
			),
			Self::IfNoneMatch => f.write_str(
				"the unwanted ETag was found (specified in If-None-Match header of your request)",
			),
			Self::InternalError => {
				f.write_str("there is an internal error that should not logically happen")
			}
			Self::NotFound => f.write_str("requested item was not found"),
			Self::WorksOnlyForDocument => f.write_str("this method works only on documents"),
			Self::WrongPath => f.write_str("the path of the item is incorrect"),
		}
	}
}
impl std::error::Error for ErrorDelete {}

#[cfg(feature = "server_bin")]
impl std::convert::From<ErrorDelete> for actix_web::HttpResponse {
	fn from(input: ErrorDelete) -> Self {
		let request_method = actix_web::http::Method::DELETE;
		match input {
			ErrorDelete::Conflict => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::CONFLICT,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorDelete::IfMatchNotFound => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorDelete::IfNoneMatch => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorDelete::InternalError => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorDelete::NotFound => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorDelete::WorksOnlyForDocument => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorDelete::WrongPath => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", input)),
				true,
			),
		}
	}
}
