pub fn put(
	storage: &dyn super::Storage,
	prefix: &str,
	path: &std::path::Path,
	if_match: &crate::Etag,
	if_none_match: &[&crate::Etag],
	item: crate::Item,
) -> crate::database::PutResult {
	match super::get(storage, prefix, path, if_match, if_none_match, true) {
		Ok(crate::Item::Document {
			content_type: old_content_type,
			content: old_content,
			..
		}) => {
			if let crate::Item::Document {
				content_type: new_content_type,
				content: new_content,
				..
			} = item
			{
				if new_content != old_content || new_content_type != old_content_type {
					match new_content {
						Some(new_content) => {
							let new_etag = crate::Etag::new();

							let serialized_data = serde_json::to_string(&crate::DataDocument {
								datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
								etag: new_etag.clone(),
								last_modified: chrono::Utc::now(),
								content_type: new_content_type,
							});

							let filedata_path = format!(
								"{}/{}/.{}.itemdata.json",
								prefix,
								path.parent().unwrap().to_str().unwrap(),
								path.file_name().unwrap().to_str().unwrap()
							);
							match serialized_data {
								Ok(serialized_data) => {
									if storage.set_item(&filedata_path, &serialized_data).is_err() {
										return crate::database::PutResult::Err(Box::new(
											PutError::GetError(super::GetError::CanNotGetStorage),
										));
									}
									if storage
										.set_item(
											&format!("{}/{}", prefix, path.to_str().unwrap()),
											&base64::encode(new_content),
										)
										.is_err()
									{
										return crate::database::PutResult::Err(Box::new(
											PutError::GetError(super::GetError::CanNotGetStorage),
										));
									}

									for ancestor in path.ancestors().skip(1) {
										let mut ancestor_path = String::new();
										if ancestor.to_str().unwrap() != "" {
											ancestor_path += ancestor.to_str().unwrap();
											if !ancestor_path.is_empty() {
												ancestor_path += "/";
											}
										}

										let folderdata_path = format!(
											"{}/{}.folder.itemdata.json",
											prefix, ancestor_path,
										);

										match storage.get_item(&folderdata_path) {
											Ok(Some(folderdata_content)) => {
												match serde_json::from_str::<crate::DataFolder>(
													&folderdata_content,
												) {
													Ok(folderdata) => {
														let mut new_folderdata = folderdata.clone();
														new_folderdata.etag = crate::Etag::new();

														match serde_json::to_string(&new_folderdata) {
														Ok(new_folderdata_content) => {
															if storage.set_item(&folderdata_path, &new_folderdata_content).is_err() {
																return crate::database::PutResult::Err(Box::new(PutError::GetError(super::GetError::CanNotGetStorage)));
															}
														}
														Err(error) => return crate::database::PutResult::Err(Box::new(PutError::CanNotSerializeFile { path: std::path::PathBuf::from(folderdata_path), error: format!("{}", error) })),
													}
													}
													Err(error) => {
														return crate::database::PutResult::Err(
															Box::new(
																PutError::CanNotDeserializeFile {
																	path: std::path::PathBuf::from(
																		folderdata_path,
																	),
																	error: format!("{}", error),
																},
															),
														)
													}
												}
											}
											Ok(None) => {
												// it should not happen
												// we can not fix it easily here because risks or conflict
												return crate::database::PutResult::Err(Box::new(
													PutError::InternalError,
												));
											}
											Err(_) => {
												return crate::database::PutResult::Err(Box::new(
													PutError::GetError(
														super::GetError::CanNotGetStorage,
													),
												));
											}
										}
									}

									return crate::database::PutResult::Updated(new_etag);
								}
								Err(error) => {
									return crate::database::PutResult::Err(Box::new(
										PutError::CanNotDeserializeFile {
											path: std::path::PathBuf::from(filedata_path),
											error: format!("{}", error),
										},
									));
								}
							}
						}
						None => {
							return crate::database::PutResult::Err(Box::new(
								PutError::NoContentInside {
									item_path: path.to_path_buf(), // TODO : not really path, but the `item` parameter of this method
								},
							));
						}
					}
				} else {
					return crate::database::PutResult::Err(Box::new(PutError::ContentNotChanged));
				}
			} else {
				return crate::database::PutResult::Err(Box::new(PutError::GetError(
					super::GetError::Conflict {
						item_path: path.to_path_buf(),
					},
				)));
			}
		}
		Ok(crate::Item::Folder { .. }) => {
			if let crate::Item::Folder { .. } = item {
				return crate::database::PutResult::Err(Box::new(PutError::DoesNotWorksForFolders));
			} else {
				return crate::database::PutResult::Err(Box::new(PutError::GetError(
					super::GetError::Conflict {
						item_path: path.to_path_buf(),
					},
				)));
			}
		}
		Err(error) => {
			let error = *error.downcast::<super::GetError>().unwrap();

			return match error {
				super::GetError::NotFound { .. } => match item {
					crate::Item::Document {
						content: Some(new_content),
						content_type: new_content_type,
						..
					} => {
						let datadocument = crate::DataDocument {
							content_type: new_content_type,
							..Default::default()
						};

						let mut parent_path = String::new();
						if let Some(parent) = path.parent() {
							let parent = parent.to_str().unwrap();
							parent_path += parent;
							if !parent.is_empty() {
								parent_path += "/";
							}
						}

						if storage
							.set_item(
								&format!(
									"{}/{}.{}.itemdata.json",
									prefix,
									parent_path,
									path.file_name().unwrap().to_str().unwrap()
								),
								&serde_json::to_string(&datadocument).unwrap(),
							)
							.is_err()
						{
							return crate::database::PutResult::Err(Box::new(PutError::GetError(
								super::GetError::CanNotGetStorage,
							)));
						}

						if storage
							.set_item(
								&format!("{}/{}", prefix, path.to_str().unwrap()),
								&base64::encode(&new_content),
							)
							.is_err()
						{
							return crate::database::PutResult::Err(Box::new(PutError::GetError(
								super::GetError::CanNotGetStorage,
							)));
						}

						for ancestor in path.ancestors().skip(1) {
							match storage.get_item(&format!(
								"{}/{}/.{}.itemdata.json",
								prefix,
								ancestor
									.parent()
									.unwrap_or_else(|| std::path::Path::new(""))
									.to_str()
									.unwrap(),
								ancestor
									.file_name()
									.unwrap_or_default()
									.to_str()
									.unwrap_or_default()
							)) {
								Ok(Some(_)) => {
									return crate::database::PutResult::Err(Box::new(
										PutError::GetError(super::GetError::Conflict {
											item_path: std::path::PathBuf::from(
												ancestor
													.to_str()
													.unwrap()
													.strip_suffix('/')
													.unwrap_or_else(|| ancestor.to_str().unwrap()),
											),
										}),
									))
								}
								Ok(None) => {
									let datafolder = crate::DataFolder::default();

									let mut ancestor_path =
										String::from(ancestor.to_str().unwrap());
									if !ancestor_path.is_empty() {
										ancestor_path += "/";
									}

									if storage
										.set_item(
											&format!(
												"{}/{}.folder.itemdata.json",
												prefix, ancestor_path,
											),
											&serde_json::to_string(&datafolder).unwrap(),
										)
										.is_err()
									{
										return crate::database::PutResult::Err(Box::new(
											PutError::GetError(super::GetError::CanNotGetStorage),
										));
									}
								}
								Err(_) => {
									return crate::database::PutResult::Err(Box::new(
										PutError::CanNotDeserializeFile {
											path: ancestor.to_path_buf(),
											error: String::new(),
										},
									))
								}
							}
						}

						crate::database::PutResult::Created(datadocument.etag)
					}
					crate::Item::Document { content: None, .. } => {
						crate::database::PutResult::Err(Box::new(PutError::NoContentInside {
							item_path: path.to_path_buf(), // TODO : not really path, but the `item` parameter of this method
						}))
					}
					crate::Item::Folder { .. } => {
						crate::database::PutResult::Err(Box::new(PutError::DoesNotWorksForFolders))
					}
				},
				_ => crate::database::PutResult::Err(Box::new(PutError::GetError(error))),
			};
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum PutError {
	GetError(super::GetError),
	DoesNotWorksForFolders,
	ContentNotChanged,
	CanNotSerializeFile {
		path: std::path::PathBuf,
		error: String,
	},
	CanNotDeserializeFile {
		path: std::path::PathBuf,
		error: String,
	},
	NoContentInside {
		item_path: std::path::PathBuf,
	},
	InternalError,
}
impl std::fmt::Display for PutError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::GetError(get_error) => std::fmt::Display::fmt(get_error, f),
			Self::DoesNotWorksForFolders => f.write_str("this method does not works on folders"),
			Self::ContentNotChanged => f.write_str("content not changed"),
			Self::CanNotSerializeFile { path, error } => f.write_fmt(format_args!(
				"can not serialize file `{}` because : {}",
				path.to_string_lossy(),
				error
			)),
			Self::CanNotDeserializeFile { path, error } => f.write_fmt(format_args!(
				"can not deserialize file `{}` because : {}",
				path.to_string_lossy(),
				error
			)),
			Self::NoContentInside { item_path } => f.write_fmt(format_args!(
				"no content found in `{}`",
				item_path.to_string_lossy()
			)),
			Self::InternalError => f.write_str("internal server error"),
		}
	}
}
impl std::error::Error for PutError {}
#[cfg(feature = "server_bin")]
impl crate::database::Error for PutError {
	fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse {
		match self {
			// TODO : we have to find a way to change method
			Self::GetError(get_error) => {
				crate::database::Error::to_response(get_error, origin, should_have_body)
			}
			Self::DoesNotWorksForFolders => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::ContentNotChanged => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::NOT_MODIFIED,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::CanNotSerializeFile { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
			Self::CanNotDeserializeFile { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
			Self::NoContentInside { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
			Self::InternalError => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(non_snake_case)]

	use super::{super::LocalStorageMock, super::Storage, put, PutError};

	// TODO : test last_modified

	fn build_test_db() -> (
		LocalStorageMock,
		String,
		crate::Etag,
		crate::Etag,
		crate::Etag,
	) {
		let AA = crate::Item::new_doc(b"AA", "text/plain");
		let A = crate::Item::new_folder(vec![("AA", AA.clone())]);
		let root = crate::Item::new_folder(vec![("A", A.clone())]);

		////////////////////////////////////////////////////////////////////////////////////////////////

		let prefix = "pontus_onyx_put_test";

		let storage = LocalStorageMock::new();

		storage.set_item(&format!("{}/", prefix), "").unwrap();

		storage
			.set_item(
				&format!("{}/.folder.itemdata.json", prefix),
				&serde_json::to_string(&crate::DataFolder {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: root.get_etag().clone(),
				})
				.unwrap(),
			)
			.unwrap();

		storage.set_item(&format!("{}/A/", prefix), "").unwrap();

		storage
			.set_item(
				&format!("{}/A/.folder.itemdata.json", prefix),
				&serde_json::to_string(&crate::DataFolder {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: A.get_etag().clone(),
				})
				.unwrap(),
			)
			.unwrap();

		if let crate::Item::Document {
			content: Some(content),
			etag: AA_etag,
			content_type: AA_content_type,
			last_modified: AA_last_modified,
		} = AA.clone()
		{
			storage
				.set_item(
					&format!("{}/A/.AA.itemdata.json", prefix),
					&serde_json::to_string(&crate::DataDocument {
						datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
						etag: AA_etag,
						content_type: AA_content_type,
						last_modified: AA_last_modified,
					})
					.unwrap(),
				)
				.unwrap();

			storage
				.set_item(&format!("{}/A/AA", prefix), &base64::encode(&content))
				.unwrap();
		} else {
			panic!()
		}

		return (
			storage,
			String::from(prefix),
			root.get_etag().clone(),
			A.get_etag().clone(),
			AA.get_etag().clone(),
		);
	}

	#[test]
	fn simple_put_on_not_existing() {
		let storage = LocalStorageMock::new();
		let prefix = String::from("pontus_onyx_put_test");

		let AA_etag = put(
			&storage,
			&prefix,
			std::path::Path::new("AA"),
			&crate::Etag::from(""),
			&[],
			crate::Item::new_doc(b"AA", "text/plain"),
		)
		.unwrap();

		assert!(storage
			.get_item(&format!("{}/.folder.itemdata.json", prefix))
			.unwrap()
			.is_some());
		let AA_datadoc: crate::DataDocument = serde_json::from_str(
			&storage
				.get_item(&format!("{}/.AA.itemdata.json", prefix))
				.unwrap()
				.unwrap(),
		)
		.unwrap();
		assert_eq!(AA_datadoc.etag, AA_etag);
		assert_eq!(AA_datadoc.content_type, "text/plain");
		assert_eq!(
			base64::decode(
				storage
					.get_item(&format!("{}/AA", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap(),
			b"AA"
		);
	}

	#[test]
	fn simple_put_on_existing() {
		let (storage, prefix, root_etag, A_etag, old_AA_etag) = build_test_db();

		let AA_etag = put(
			&storage,
			&prefix,
			std::path::Path::new("A/AA"),
			&crate::Etag::from(""),
			&[],
			crate::Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap();

		assert_ne!(old_AA_etag, AA_etag);

		assert_ne!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			root_etag
		);

		assert!(storage
			.get_item(&format!("{}/.A.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert_ne!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			A_etag
		);

		assert!(storage
			.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		let AA_datadoc: crate::DataDocument = serde_json::from_str(
			&storage
				.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
				.unwrap()
				.unwrap(),
		)
		.unwrap();
		assert_eq!(AA_datadoc.etag, AA_etag);
		assert_eq!(AA_datadoc.content_type, "text/plain2");
		assert_eq!(
			base64::decode(
				storage
					.get_item(&format!("{}/A/AA", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap(),
			b"AA2"
		);
	}

	#[test]
	fn content_not_changed() {
		let (storage, prefix, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			*put(
				&storage,
				&prefix,
				std::path::Path::new("A/AA"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"AA", "text/plain")
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::ContentNotChanged
		);

		assert_eq!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			root_etag
		);

		assert!(storage
			.get_item(&format!("{}/.A.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert_eq!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			A_etag
		);

		assert!(storage
			.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		let AA_datadoc: crate::DataDocument = serde_json::from_str(
			&storage
				.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
				.unwrap()
				.unwrap(),
		)
		.unwrap();
		assert_eq!(AA_datadoc.etag, AA_etag);
		assert_eq!(AA_datadoc.content_type, "text/plain");
		assert_eq!(
			base64::decode(
				storage
					.get_item(&format!("{}/A/AA", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap(),
			b"AA"
		);
	}

	#[test]
	fn does_not_works_for_folders() {
		let prefix = "pontus_onyx_get_test";
		let storage = LocalStorageMock::new();

		assert_eq!(
			*put(
				&storage,
				&prefix,
				std::path::Path::new(""),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_folder(vec![])
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::DoesNotWorksForFolders
		);

		assert_eq!(storage.length().unwrap(), 0);
	}

	#[test]
	fn put_with_if_none_match_all_on_not_existing() {
		let storage = LocalStorageMock::new();
		let prefix = String::from("pontus_onyx_put_test");

		let AA_etag = put(
			&storage,
			&prefix,
			std::path::Path::new("A/AA"),
			&crate::Etag::from(""),
			&[&crate::Etag::from("*")],
			crate::Item::new_doc(b"AA", "text/plain"),
		)
		.unwrap();

		assert!(serde_json::from_str::<crate::DataFolder>(
			&storage
				.get_item(&format!("{}/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.is_ok());

		assert!(storage
			.get_item(&format!("{}/.A.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert!(serde_json::from_str::<crate::DataFolder>(
			&storage
				.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.is_ok());

		assert!(storage
			.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		let AA_datadoc: crate::DataDocument = serde_json::from_str(
			&storage
				.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
				.unwrap()
				.unwrap(),
		)
		.unwrap();
		assert_eq!(AA_datadoc.etag, AA_etag);
		assert_eq!(AA_datadoc.content_type, "text/plain");
		assert_eq!(
			base64::decode(
				storage
					.get_item(&format!("{}/A/AA", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap(),
			b"AA"
		);
	}

	#[test]
	fn put_with_if_none_match_all_on_existing() {
		let (storage, prefix, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			*put(
				&storage,
				&prefix,
				std::path::Path::new("A/AA"),
				&crate::Etag::from(""),
				&[&crate::Etag::from("*")],
				crate::Item::new_doc(b"AA2", "text/plain2"),
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(super::super::GetError::IfNoneMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				found: AA_etag.clone(),
				search: crate::Etag::from("*")
			})
		);

		assert_eq!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			root_etag
		);

		assert!(storage
			.get_item(&format!("{}/.A.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert_eq!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			A_etag
		);

		assert!(storage
			.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		let AA_datadoc: crate::DataDocument = serde_json::from_str(
			&storage
				.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
				.unwrap()
				.unwrap(),
		)
		.unwrap();
		assert_eq!(AA_datadoc.etag, AA_etag);
		assert_eq!(AA_datadoc.content_type, "text/plain");
		assert_eq!(
			base64::decode(
				storage
					.get_item(&format!("{}/A/AA", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap(),
			b"AA"
		);
	}

	#[test]
	fn put_with_if_match_not_found() {
		let (storage, prefix, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			*put(
				&storage,
				&prefix,
				std::path::Path::new("A/AA"),
				&crate::Etag::from("ANOTHER_ETAG"),
				&[],
				crate::Item::new_doc(b"AA2", "text/plain2"),
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(super::super::GetError::NoIfMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				found: AA_etag.clone(),
				search: crate::Etag::from("ANOTHER_ETAG")
			})
		);

		assert_eq!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			root_etag
		);

		assert!(storage
			.get_item(&format!("{}/.A.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert_eq!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			A_etag
		);

		assert!(storage
			.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		let AA_datadoc: crate::DataDocument = serde_json::from_str(
			&storage
				.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
				.unwrap()
				.unwrap(),
		)
		.unwrap();
		assert_eq!(AA_datadoc.etag, AA_etag);
		assert_eq!(AA_datadoc.content_type, "text/plain");
		assert_eq!(
			base64::decode(
				storage
					.get_item(&format!("{}/A/AA", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap(),
			b"AA"
		);
	}

	#[test]
	fn put_with_if_match_found() {
		let (storage, prefix, root_etag, A_etag, mut AA_etag) = build_test_db();

		AA_etag = put(
			&storage,
			&prefix,
			std::path::Path::new("A/AA"),
			&AA_etag,
			&[],
			crate::Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap();

		assert_ne!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			root_etag
		);

		assert!(storage
			.get_item(&format!("{}/.A.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert_ne!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			A_etag
		);

		assert!(storage
			.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		let AA_datadoc: crate::DataDocument = serde_json::from_str(
			&storage
				.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
				.unwrap()
				.unwrap(),
		)
		.unwrap();
		assert_eq!(AA_datadoc.etag, AA_etag);
		assert_eq!(AA_datadoc.content_type, "text/plain2");
		assert_eq!(
			base64::decode(
				storage
					.get_item(&format!("{}/A/AA", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap(),
			b"AA2"
		);
	}

	#[test]
	fn put_with_if_match_all() {
		let (storage, prefix, root_etag, A_etag, old_AA_etag) = build_test_db();

		let AA_etag = put(
			&storage,
			&prefix,
			std::path::Path::new("A/AA"),
			&crate::Etag::from("*"),
			&[],
			crate::Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap();

		assert_ne!(old_AA_etag, AA_etag);

		assert_ne!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			root_etag
		);

		assert!(storage
			.get_item(&format!("{}/.A.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert_ne!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			A_etag
		);

		assert!(storage
			.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		let AA_datadoc: crate::DataDocument = serde_json::from_str(
			&storage
				.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
				.unwrap()
				.unwrap(),
		)
		.unwrap();
		assert_eq!(AA_datadoc.etag, AA_etag);
		assert_eq!(AA_datadoc.content_type, "text/plain2");
		assert_eq!(
			base64::decode(
				storage
					.get_item(&format!("{}/A/AA", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap(),
			b"AA2"
		);
	}

	#[test]
	fn put_with_existing_document_conflict() {
		let (storage, prefix, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			*put(
				&storage,
				&prefix,
				std::path::Path::new("A/AA/AAA"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"AAA", "text/plain")
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(super::super::GetError::Conflict {
				item_path: std::path::PathBuf::from("A/AA")
			})
		);

		assert_eq!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			root_etag
		);

		assert!(storage
			.get_item(&format!("{}/.A.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert_eq!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			A_etag
		);

		assert!(storage
			.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		let AA_datadoc: crate::DataDocument = serde_json::from_str(
			&storage
				.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
				.unwrap()
				.unwrap(),
		)
		.unwrap();
		assert_eq!(AA_datadoc.etag, AA_etag);
		assert_eq!(AA_datadoc.content_type, "text/plain");
		assert_eq!(
			base64::decode(
				storage
					.get_item(&format!("{}/A/AA", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap(),
			b"AA"
		);
	}

	#[test]
	fn put_with_existing_folder_conflict() {
		let (storage, prefix, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			*put(
				&storage,
				&prefix,
				std::path::Path::new("A"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"A", "text/plain")
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(super::super::GetError::Conflict {
				item_path: std::path::PathBuf::from("A")
			})
		);

		assert_eq!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			root_etag
		);

		assert!(storage
			.get_item(&format!("{}/.A.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert_eq!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			A_etag
		);

		assert!(storage
			.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		let AA_datadoc: crate::DataDocument = serde_json::from_str(
			&storage
				.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
				.unwrap()
				.unwrap(),
		)
		.unwrap();
		assert_eq!(AA_datadoc.etag, AA_etag);
		assert_eq!(AA_datadoc.content_type, "text/plain");
		assert_eq!(
			base64::decode(
				storage
					.get_item(&format!("{}/A/AA", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap(),
			b"AA"
		);
	}

	#[test]
	fn put_in_public() {
		let storage = LocalStorageMock::new();
		let prefix = String::from("pontus_onyx_put_test");

		let AA_etag = put(
			&storage,
			&prefix,
			std::path::Path::new("public/A/AA"),
			&crate::Etag::from(""),
			&[],
			crate::Item::new_doc(b"AA", "text/plain"),
		)
		.unwrap();

		assert!(serde_json::from_str::<crate::DataFolder>(
			&storage
				.get_item(&format!("{}/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.is_ok());

		assert!(storage
			.get_item(&format!("{}/.public.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert!(serde_json::from_str::<crate::DataFolder>(
			&storage
				.get_item(&format!("{}/public/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.is_ok());

		assert!(storage
			.get_item(&format!("{}/public/.A.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert!(serde_json::from_str::<crate::DataFolder>(
			&storage
				.get_item(&format!("{}/public/A/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.is_ok());

		assert!(storage
			.get_item(&format!("{}/public/A/AA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		let AA_datadoc: crate::DataDocument = serde_json::from_str(
			&storage
				.get_item(&format!("{}/public/A/.AA.itemdata.json", prefix))
				.unwrap()
				.unwrap(),
		)
		.unwrap();
		assert_eq!(AA_datadoc.etag, AA_etag);
		assert_eq!(AA_datadoc.content_type, "text/plain");
		assert_eq!(
			base64::decode(
				storage
					.get_item(&format!("{}/public/A/AA", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap(),
			b"AA"
		);
	}

	#[test]
	fn put_in_incorrect_path() {
		let storage = LocalStorageMock::new();
		let prefix = String::from("pontus_onyx_put_test");

		assert_eq!(
			*put(
				&storage,
				&prefix,
				std::path::Path::new("A/../AA"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"AA", "text/plain"),
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(super::super::GetError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/../"),
				error: String::from("`..` is not allowed")
			})
		);

		assert_eq!(storage.length().unwrap(), 0);
	}
}
