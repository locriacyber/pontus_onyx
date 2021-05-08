impl super::Database {
	pub fn put(
		&mut self,
		path: &str,
		content: crate::Item,
		if_match: Option<&str>,
		if_none_match: Option<Vec<&str>>,
	) -> PutResult {
		match &content {
			crate::Item::Document {
				content: document_content,
				content_type: document_content_type,
				..
			} => {
				/*
				TODO :
					* its version being updated, as well as that of its parent folder
						and further ancestor folders, using a strong validator [HTTP,
						section 7.2].
				*/
				match self.get(&path, None, if_none_match) {
					Ok(crate::Item::Document {
						etag: document_etag,
						..
					}) => {
						let if_match_result = if let Some(find_match) = if_match {
							let find_match = find_match.trim().replace('"', "");

							document_etag == &find_match
						} else {
							true
						};

						if !if_match_result {
							return PutResult::Err(PutError::IfMatchNotFound);
						}

						let mut new_content = content.clone();
						if let crate::Item::Document {
							etag,
							last_modified,
							..
						} = &mut new_content
						{
							*etag = ulid::Ulid::new().to_string();
							*last_modified = chrono::Utc::now();
						}

						// update :

						let paths: Vec<&str> = path.split('/').collect();

						match new_content {
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
														&mut paths
															.iter()
															.cloned()
															.take(paths.len()),
													) {
														Ok(()) => PutResult::Updated(new_etag),
														Err(e) => {
															// TODO : is following conversion is OK ?
															PutResult::Err(match e {
																super::UpdateFoldersEtagsError::FolderDocumentConflict => PutError::Conflict,
																super::UpdateFoldersEtagsError::MissingFolder => PutError::NotFound,
																super::UpdateFoldersEtagsError::WrongFolderName => PutError::WrongPath,
															})
														}
													}
												} else {
													PutResult::Err(PutError::NotModified)
												}
											} else {
												PutResult::Err(PutError::Conflict)
											}
										}
										Ok(None) => PutResult::Err(PutError::NotFound),
										Err(super::FetchError::FolderDocumentConflict) => {
											PutResult::Err(PutError::Conflict)
										}
									}
								} else {
									PutResult::Err(PutError::WrongPath)
								}
							}
							crate::Item::Folder {
								etag: _,
								content: _,
							} => PutResult::Err(PutError::WorksOnlyForDocument),
						}
					}
					Ok(crate::Item::Folder { .. }) => {
						return PutResult::Err(PutError::Conflict);
					}
					Err(super::get::GetError::NotFound) => {
						// create :

						let paths: Vec<&str> = path.split('/').collect();

						if paths.iter().all(|e| super::path::is_ok(e, false)) {
							if let crate::Item::Folder {
								etag: _,
								content: root_folder_content,
							} = &mut self.content
							{
								match Self::build_folders(
									root_folder_content,
									&mut paths.iter().cloned().take(paths.len() - 1),
								) {
									Ok(()) => {
										match self.fetch_item_mut(&paths) {
											Ok(Some(_e)) => PutResult::Err(PutError::InternalError), // should never happen
											Ok(None) => {
												let folder_path: Vec<&str> = paths
													.iter()
													.take(paths.len() - 1)
													.cloned()
													.collect();

												match self.fetch_item_mut(&folder_path) {
													Ok(Some(crate::Item::Folder {
														etag: _,
														content: folder_content,
													})) => {
														let etag = ulid::Ulid::new().to_string();

														// TODO : check content of paths.last()
														folder_content.insert(
															String::from(*paths.last().unwrap()),
															Box::new(crate::Item::Document {
																etag: etag.clone(),
																content: document_content.to_vec(),
																content_type: String::from(
																	document_content_type,
																),
																last_modified: chrono::Utc::now(),
															}),
														);

														match Self::update_folders_etags(
															&mut self.content,
															&mut paths
																.iter()
																.cloned()
																.take(paths.len() - 1),
														) {
															Ok(()) => PutResult::Created(etag),
															Err(e) => {
																// TODO : is following conversion is OK ?
																PutResult::Err(match e {
																	super::UpdateFoldersEtagsError::FolderDocumentConflict => PutError::Conflict,
																	super::UpdateFoldersEtagsError::MissingFolder => PutError::NotFound,
																	super::UpdateFoldersEtagsError::WrongFolderName => PutError::WrongPath,
																})
															}
														}
													}
													_ => todo!(),
												}
											}
											Err(super::FetchError::FolderDocumentConflict) => {
												PutResult::Err(PutError::Conflict)
											}
										}
									}
									Err(e) => {
										// TODO : is following conversion is OK ?
										PutResult::Err(match e {
											super::FolderBuildError::FolderDocumentConflict => {
												PutError::Conflict
											}
											super::FolderBuildError::WrongFolderName => {
												PutError::WrongPath
											}
										})
									}
								}
							} else {
								PutResult::Err(PutError::InternalError)
							}
						} else {
							PutResult::Err(PutError::WrongPath)
						}
					}
					Err(super::get::GetError::IfNoneMatch) => {
						return PutResult::Err(PutError::IfNoneMatch);
					}
					Err(super::get::GetError::IfMatchNotFound) => {
						return PutResult::Err(PutError::InternalError); // should never happen
					}
					Err(super::get::GetError::CanNotBeListed) => {
						return PutResult::Err(PutError::WorksOnlyForDocument);
					}
					Err(super::get::GetError::WrongPath) => {
						return PutResult::Err(PutError::WrongPath);
					}
					Err(super::get::GetError::Conflict) => {
						return PutResult::Err(PutError::Conflict);
					}
				}
			}
			crate::Item::Folder { .. } => {
				return PutResult::Err(PutError::WorksOnlyForDocument);
			}
		}
	}
}

#[derive(Debug)]
pub enum PutError {
	Conflict,
	IfMatchNotFound,
	IfNoneMatch,
	InternalError,
	NotFound,
	NotModified,
	WorksOnlyForDocument,
	WrongPath,
}
impl std::fmt::Display for PutError {
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
			Self::NotModified => f.write_str("this document was not modified"),
			Self::WorksOnlyForDocument => f.write_str("this method works only on documents"),
			Self::WrongPath => f.write_str("the path of the item is incorrect"),
		}
	}
}
impl std::error::Error for PutError {}

#[cfg(feature = "server")]
impl std::convert::Into<actix_web::HttpResponse> for PutError {
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
			Self::NotModified => crate::utils::build_http_json_response(
				actix_web::http::StatusCode::NOT_MODIFIED,
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

pub enum PutResult {
	Created(String),
	Updated(String),
	Err(PutError),
}
