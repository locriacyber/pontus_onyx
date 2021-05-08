impl super::Database {
	pub fn delete(&mut self, path: &str, if_match: Option<&str>) -> Result<String, DeleteError> {
		let if_match_result = if let Some(find_match) = if_match {
			let find_match = find_match.trim().replace('"', "");

			match self.get(path, if_match, None) {
				Ok(crate::Item::Document {
					etag: document_etag,
					..
				}) => Ok(document_etag == &find_match),
				Ok(crate::Item::Folder { .. }) => Err(DeleteError::WorksOnlyForDocument),
				Err(crate::GetError::CanNotBeListed) => Err(DeleteError::WorksOnlyForDocument),
				Err(crate::GetError::Conflict) => Err(DeleteError::Conflict),
				Err(crate::GetError::IfMatchNotFound) => Err(DeleteError::IfMatchNotFound),
				Err(crate::GetError::IfNoneMatch) => Err(DeleteError::IfNoneMatch),
				Err(crate::GetError::NotFound) => Err(DeleteError::NotFound),
				Err(crate::GetError::WrongPath) => Err(DeleteError::WrongPath),
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
															super::UpdateFoldersEtagsError::FolderDocumentConflict => DeleteError::Conflict,
															super::UpdateFoldersEtagsError::MissingFolder => DeleteError::NotFound,
															super::UpdateFoldersEtagsError::WrongFolderName => DeleteError::WrongPath,
														})
													}
												}
											}
											None => Err(DeleteError::NotFound),
										}
									} else {
										Err(DeleteError::NotFound)
									}
								}
								Ok(crate::Item::Folder {
									etag: _,
									content: _,
								}) => Err(DeleteError::WorksOnlyForDocument),
								Err(super::get::GetError::CanNotBeListed) => {
									Err(DeleteError::WorksOnlyForDocument)
								} // TODO : is this OK ?
								Err(super::get::GetError::Conflict) => Err(DeleteError::Conflict),
								Err(super::get::GetError::IfMatchNotFound) => {
									Err(DeleteError::IfMatchNotFound)
								}
								Err(super::get::GetError::IfNoneMatch) => {
									Err(DeleteError::InternalError)
								} // should never happen
								Err(super::get::GetError::NotFound) => Err(DeleteError::NotFound),
								Err(super::get::GetError::WrongPath) => Err(DeleteError::WrongPath),
							}
						} else {
							Err(DeleteError::WorksOnlyForDocument)
						}
					} else {
						Err(DeleteError::WrongPath)
					}
				} else {
					return Err(DeleteError::IfMatchNotFound);
				}
			}
			Err(e) => {
				return Err(e);
			}
		}
	}
}

#[derive(Debug)]
pub enum DeleteError {
	Conflict,
	IfMatchNotFound,
	IfNoneMatch,
	InternalError,
	NotFound,
	WorksOnlyForDocument,
	WrongPath,
}
impl std::fmt::Display for DeleteError {
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
impl std::error::Error for DeleteError {}

#[cfg(feature = "server")]
impl std::convert::Into<actix_web::HttpResponse> for DeleteError {
	fn into(self) -> actix_web::HttpResponse {
		match self {
			Self::Conflict => crate::utils::build_http_json_response(
				actix_web::http::StatusCode::CONFLICT,
				None,
				Some(format!("{}", self)),
				true,
			),
			Self::IfMatchNotFound => crate::utils::build_http_json_response(
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", self)),
				true,
			),
			Self::IfNoneMatch => crate::utils::build_http_json_response(
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", self)),
				true,
			),
			Self::InternalError => crate::utils::build_http_json_response(
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				Some(format!("{}", self)),
				true,
			),
			Self::NotFound => crate::utils::build_http_json_response(
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!("{}", self)),
				true,
			),
			Self::WorksOnlyForDocument => crate::utils::build_http_json_response(
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", self)),
				true,
			),
			Self::WrongPath => crate::utils::build_http_json_response(
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", self)),
				true,
			),
		}
	}
}
