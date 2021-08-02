pub fn delete(
	storage: &dyn super::Storage,
	prefix: &str,
	path: &std::path::Path,
	if_match: &crate::Etag,
) -> Result<crate::Etag, Box<dyn std::error::Error>> {
	if path.to_str().unwrap().ends_with('/') || path.to_str().unwrap().ends_with('\\') {
		return Err(Box::new(DeleteError::DoesNotWorksForFolders));
	}

	match super::get(storage, prefix, path, if_match, &[], false) {
		Ok(crate::Item::Document { etag: old_etag, .. }) => {
			let parent = path.parent().unwrap_or_else(|| std::path::Path::new(""));

			let file_path = format!(
				"{}/.{}.itemdata.json",
				parent.to_str().unwrap(),
				path.file_name().unwrap().to_str().unwrap()
			);
			if storage
				.remove_item(&format!("{}/{}", prefix, file_path))
				.is_err()
			{
				return Err(Box::new(DeleteError::CanNotWriteFile {
					path: std::path::PathBuf::from(&file_path),
					error: String::new(),
				}));
			}

			let file_path = path.file_name().unwrap().to_str().unwrap();
			if storage
				.remove_item(&format!("{}/{}", prefix, file_path))
				.is_err()
			{
				return Err(Box::new(DeleteError::CanNotWriteFile {
					path: std::path::PathBuf::from(&file_path),
					error: String::new(),
				}));
			}

			if storage
				.remove_item(&format!("{}/{}", prefix, &path.to_str().unwrap()))
				.is_err()
			{
				return Err(Box::new(DeleteError::CanNotWriteFile {
					path: std::path::PathBuf::from(path),
					error: String::new(),
				}));
			}

			for parent in path.ancestors().skip(1) {
				let mut has_childs = false;
				for i in 0..storage.length().unwrap_or_default() {
					let key = storage.key(i).unwrap_or_default().unwrap_or_default();
					if key.starts_with(&format!("{}/{}", prefix, parent.to_str().unwrap()))
						&& !key.ends_with(".itemdata.json")
					{
						has_childs = true;
						break;
					}
				}
				let mut parent_as_str = String::from(parent.to_str().unwrap());
				parent_as_str =
					String::from(parent_as_str.strip_suffix('/').unwrap_or(&parent_as_str));
				if !parent_as_str.is_empty() {
					parent_as_str = format!("{}/", parent_as_str);
				}
				let folder_itemdata_path =
					format!("{}/{}.folder.itemdata.json", prefix, parent_as_str);

				if !has_childs {
					if storage.remove_item(&folder_itemdata_path).is_err() {
						return Err(Box::new(DeleteError::CanNotWriteFile {
							path: std::path::PathBuf::from(&parent),
							error: String::new(),
						}));
					}

				// TODO : eventually cleanup folder content ? (eventually remaining *.itemdata.* files, but should not)
				} else {
					let folder_itemdata_content = storage.get_item(&folder_itemdata_path);

					match folder_itemdata_content {
						Ok(Some(folder_itemdata_content)) => {
							match serde_json::from_str::<crate::DataFolder>(
								&folder_itemdata_content,
							) {
								Ok(folder_itemdata) => {
									let mut new_folder_itemdata = folder_itemdata.clone();
									new_folder_itemdata.datastruct_version =
										String::from(env!("CARGO_PKG_VERSION"));
									new_folder_itemdata.etag = crate::Etag::new();

									match serde_json::to_string(&new_folder_itemdata) {
										Ok(new_folder_itemdata_content) => {
											if storage
												.set_item(
													&folder_itemdata_path,
													&new_folder_itemdata_content,
												)
												.is_err()
											{
												return Err(Box::new(
													DeleteError::CanNotWriteFile {
														path: std::path::PathBuf::from(
															&folder_itemdata_path,
														),
														error: String::new(),
													},
												));
											}
										}
										Err(error) => {
											return Err(Box::new(
												DeleteError::CanNotSerializeFile {
													path: std::path::PathBuf::from(
														&folder_itemdata_path,
													),
													error: format!("{}", error),
												},
											));
										}
									}
								}
								Err(error) => {
									return Err(Box::new(DeleteError::CanNotDeserializeFile {
										path: std::path::PathBuf::from(&folder_itemdata_path),
										error: format!("{}", error),
									}));
								}
							}
						}
						Ok(None) => match serde_json::to_string(&crate::DataFolder::default()) {
							Ok(new_folder_itemdata_content) => {
								if storage
									.set_item(&folder_itemdata_path, &new_folder_itemdata_content)
									.is_err()
								{
									return Err(Box::new(DeleteError::CanNotWriteFile {
										path: std::path::PathBuf::from(&folder_itemdata_path),
										error: String::new(),
									}));
								}
							}
							Err(error) => {
								return Err(Box::new(DeleteError::CanNotSerializeFile {
									path: std::path::PathBuf::from(&folder_itemdata_path),
									error: format!("{}", error),
								}));
							}
						},
						Err(_) => {
							return Err(Box::new(DeleteError::GetError(
								super::GetError::CanNotGetStorage,
							)));
						}
					}
				}
			}

			Ok(old_etag)
		}
		Ok(crate::Item::Folder { .. }) => Err(Box::new(DeleteError::DoesNotWorksForFolders)),
		Err(error) => Err(Box::new(DeleteError::GetError(*error.downcast().unwrap()))),
	}
}

#[derive(Debug, PartialEq)]
pub enum DeleteError {
	GetError(super::GetError),
	DoesNotWorksForFolders,
	CanNotDelete {
		path: std::path::PathBuf,
		error: String,
	},
	CanNotReadFile {
		path: std::path::PathBuf,
		error: String,
	},
	CanNotWriteFile {
		path: std::path::PathBuf,
		error: String,
	},
	CanNotSerializeFile {
		path: std::path::PathBuf,
		error: String,
	},
	CanNotDeserializeFile {
		path: std::path::PathBuf,
		error: String,
	},
}
impl std::fmt::Display for DeleteError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::GetError(get_error) => std::fmt::Display::fmt(get_error, f),
			Self::DoesNotWorksForFolders => f.write_str("this method does not works for folders"),
			Self::CanNotDelete { path, error } => f.write_fmt(format_args!(
				"can not delete file `{}` because : {}",
				path.to_string_lossy(),
				error
			)),
			Self::CanNotReadFile { path, error } => f.write_fmt(format_args!(
				"can not read file `{}` because : {}",
				path.to_string_lossy(),
				error
			)),
			Self::CanNotWriteFile { path, error } => f.write_fmt(format_args!(
				"can not write file `{}` because : {}",
				path.to_string_lossy(),
				error
			)),
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
		}
	}
}
impl std::error::Error for DeleteError {}
#[cfg(feature = "server_bin")]
impl crate::database::Error for DeleteError {
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
			Self::CanNotDelete { path: _, error: _ } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
			Self::CanNotReadFile { path: _, error: _ } => {
				crate::database::build_http_json_response(
					origin,
					&actix_web::http::Method::PUT,
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
					None,
					should_have_body,
				)
			}
			Self::CanNotWriteFile { path: _, error: _ } => {
				crate::database::build_http_json_response(
					origin,
					&actix_web::http::Method::PUT,
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
					None,
					should_have_body,
				)
			}
			Self::CanNotSerializeFile { path: _, error: _ } => {
				crate::database::build_http_json_response(
					origin,
					&actix_web::http::Method::PUT,
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
					None,
					should_have_body,
				)
			}
			Self::CanNotDeserializeFile { path: _, error: _ } => {
				crate::database::build_http_json_response(
					origin,
					&actix_web::http::Method::PUT,
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
					None,
					should_have_body,
				)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(non_snake_case)]

	use super::{super::GetError, super::LocalStorageMock, super::Storage, delete, DeleteError};

	// TODO : test last_modified

	fn build_test_db() -> (
		LocalStorageMock,
		String,
		crate::Etag,
		crate::Etag,
		crate::Etag,
		crate::Etag,
		crate::Etag,
		crate::Etag,
		crate::Etag,
		crate::Etag,
	) {
		let AAA = crate::Item::new_doc(b"AAA", "text/plain");
		let AA = crate::Item::new_folder(vec![("AAA", AAA.clone())]);
		let AB = crate::Item::new_doc(b"AB", "text/plain");
		let A = crate::Item::new_folder(vec![("AA", AA.clone()), ("AB", AB.clone())]);

		let BA = crate::Item::new_doc(b"BA", "text/plain");
		let B = crate::Item::new_folder(vec![("BA", BA.clone())]);
		let public = crate::Item::new_folder(vec![("B", B.clone())]);

		let root = crate::Item::new_folder(vec![("A", A.clone()), ("public", public.clone())]);

		////////////////////////////////////////////////////////////////////////////////////////////////

		let prefix = String::from("pontus_onyx_delete_test");

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

		/*
		storage.set_item(&format!("{}/A/", prefix), "").unwrap();
		*/

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

		storage
			.set_item(
				&format!("{}/A/AA/.folder.itemdata.json", prefix),
				&serde_json::to_string(&crate::DataFolder {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: AA.get_etag().clone(),
				})
				.unwrap(),
			)
			.unwrap();

		if let crate::Item::Document {
			content: Some(content),
			etag: AAA_etag,
			content_type: AAA_content_type,
			last_modified: AAA_last_modified,
		} = AAA.clone()
		{
			storage
				.set_item(
					&format!("{}/A/AA/.AAA.itemdata.json", prefix),
					&serde_json::to_string(&crate::DataDocument {
						datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
						etag: AAA_etag,
						content_type: AAA_content_type,
						last_modified: AAA_last_modified,
					})
					.unwrap(),
				)
				.unwrap();

			storage
				.set_item(&format!("{}/A/AA/AAA", prefix), &base64::encode(&content))
				.unwrap();
		} else {
			panic!();
		}

		if let crate::Item::Document {
			content: Some(content),
			etag: AB_etag,
			content_type: AB_content_type,
			last_modified: AB_last_modified,
		} = AB.clone()
		{
			storage
				.set_item(
					&format!("{}/A/.AB.itemdata.json", prefix),
					&serde_json::to_string(&crate::DataDocument {
						datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
						etag: AB_etag,
						content_type: AB_content_type,
						last_modified: AB_last_modified,
					})
					.unwrap(),
				)
				.unwrap();

			storage
				.set_item(&format!("{}/A/AB", prefix), &base64::encode(&content))
				.unwrap();
		} else {
			panic!();
		}

		/*
		storage
			.set_item(&format!("{}/public/", prefix), "")
			.unwrap();
		*/

		storage
			.set_item(
				&format!("{}/public/.folder.itemdata.json", prefix),
				&serde_json::to_string(&crate::DataFolder {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: public.get_etag().clone(),
				})
				.unwrap(),
			)
			.unwrap();

		/*
		storage.set_item(&format!("{}/public/B/", prefix), "").unwrap();
		*/

		storage
			.set_item(
				&format!("{}/public/B/.folder.itemdata.json", prefix),
				&serde_json::to_string(&crate::DataFolder {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: B.get_etag().clone(),
				})
				.unwrap(),
			)
			.unwrap();

		if let crate::Item::Document {
			content: Some(content),
			etag: BA_etag,
			content_type: BA_content_type,
			last_modified: BA_last_modified,
		} = BA.clone()
		{
			storage
				.set_item(
					&format!("{}/public/B/.BA.itemdata.json", prefix),
					&serde_json::to_string(&crate::DataDocument {
						datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
						etag: BA_etag,
						content_type: BA_content_type,
						last_modified: BA_last_modified,
					})
					.unwrap(),
				)
				.unwrap();

			storage
				.set_item(
					&format!("{}/public/B/BA", prefix),
					&base64::encode(&content),
				)
				.unwrap();
		} else {
			panic!();
		}

		for i in 0..storage.length().unwrap() {
			dbg!(&storage.key(i).unwrap().unwrap());
		}

		return (
			storage,
			prefix,
			root.get_etag().clone(),
			A.get_etag().clone(),
			AA.get_etag().clone(),
			AB.get_etag().clone(),
			AAA.get_etag().clone(),
			public.get_etag().clone(),
			B.get_etag().clone(),
			BA.get_etag().clone(),
		);
	}

	#[test]
	fn simple_delete_on_not_existing() {
		let storage = LocalStorageMock::new();
		let prefix = String::from("pontus_onyx_delete_test");
		storage
			.set_item(
				&format!("{}/.folder.itemdata.json", &prefix),
				&serde_json::to_string(&crate::DataFolder::default()).unwrap(),
			)
			.unwrap();

		for i in 0..storage.length().unwrap() {
			dbg!(&storage.key(i).unwrap().unwrap());
		}

		assert_eq!(
			*delete(
				&storage,
				&prefix,
				std::path::Path::new("A/AA/AAA"),
				&crate::Etag::from(""),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::GetError(GetError::NotFound {
				item_path: std::path::PathBuf::from("A/")
			})
		);

		assert_eq!(storage.length().unwrap(), 1);
		assert_eq!(
			storage.key(0).unwrap().unwrap(),
			format!("{}/.folder.itemdata.json", &prefix)
		);
	}

	#[test]
	fn simple_delete_on_existing() {
		let (storage, prefix, root_etag, A_etag, _, _, AAA_etag, _, _, _) = build_test_db();

		let old_AAA_etag = delete(
			&storage,
			&prefix,
			std::path::Path::new("A/AA/AAA"),
			&crate::Etag::from(""),
		)
		.unwrap();

		assert_eq!(AAA_etag, old_AAA_etag);

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
		assert!(storage
			.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
			.unwrap()
			.is_none());

		assert!(storage
			.get_item(&format!("{}/A/AA/AAA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert!(storage
			.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert!(storage
			.get_item(&format!("{}/A/AA/AAA/", prefix))
			.unwrap()
			.is_none());
		assert!(storage
			.get_item(&format!("{}/A/AA/AAA", prefix))
			.unwrap()
			.is_none());

		assert!(storage
			.get_item(&format!("{}/A/.AB.itemdata.json", prefix))
			.unwrap()
			.is_some());
		assert!(storage
			.get_item(&format!("{}/A/AB/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
	}

	#[test]
	fn does_not_works_for_folders() {
		let (storage, prefix, root_etag, A_etag, AA_etag, _, AAA_etag, _, _, _) = build_test_db();

		assert_eq!(
			*delete(
				&storage,
				&prefix,
				std::path::Path::new("A/AA/"),
				&crate::Etag::from(""),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::DoesNotWorksForFolders,
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
			.is_some());
		assert_eq!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			AA_etag
		);

		assert!(storage
			.get_item(&format!("{}/A/AA/AAA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert!(storage
			.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
			.unwrap()
			.is_some());
		assert_eq!(
			serde_json::from_str::<crate::DataDocument>(
				&storage
					.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			AAA_etag
		);
		assert_eq!(
			base64::decode(
				storage
					.get_item(&format!("{}/A/AA/AAA", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap(),
			b"AAA"
		);

		assert!(storage
			.get_item(&format!("{}/A/.AB.itemdata.json", prefix))
			.unwrap()
			.is_some());
		assert!(storage
			.get_item(&format!("{}/A/AB/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
	}

	#[test]
	fn delete_with_if_match_not_found() {
		let (storage, prefix, root_etag, A_etag, AA_etag, _, AAA_etag, _, _, _) = build_test_db();

		assert_eq!(
			*delete(
				&storage,
				&prefix,
				std::path::Path::new("A/AA/AAA"),
				&crate::Etag::from("OTHER_ETAG"),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::GetError(GetError::NoIfMatch {
				item_path: std::path::PathBuf::from("A/AA/AAA"),
				found: AAA_etag.clone(),
				search: crate::Etag::from("OTHER_ETAG")
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
			.is_some());
		assert_eq!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			AA_etag
		);

		assert!(storage
			.get_item(&format!("{}/A/AA/AAA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert!(storage
			.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
			.unwrap()
			.is_some());
		assert_eq!(
			serde_json::from_str::<crate::DataDocument>(
				&storage
					.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			AAA_etag
		);
		assert_eq!(
			base64::decode(
				storage
					.get_item(&format!("{}/A/AA/AAA", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap(),
			b"AAA"
		);

		assert!(storage
			.get_item(&format!("{}/A/.AB.itemdata.json", prefix))
			.unwrap()
			.is_some());
		assert!(storage
			.get_item(&format!("{}/A/AB/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
	}

	#[test]
	fn delete_with_if_match_found() {
		let (storage, prefix, root_etag, A_etag, _, _, AAA_etag, _, _, _) = build_test_db();

		let old_AAA_etag = delete(
			&storage,
			&prefix,
			std::path::Path::new("A/AA/AAA"),
			&AAA_etag,
		)
		.unwrap();

		assert_eq!(old_AAA_etag, AAA_etag);

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
		assert!(storage
			.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
			.unwrap()
			.is_none());

		assert!(storage
			.get_item(&format!("{}/A/AA/AAA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert!(storage
			.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert!(storage
			.get_item(&format!("{}/A/AA/AAA/", prefix))
			.unwrap()
			.is_none());
		assert!(storage
			.get_item(&format!("{}/A/AA/AAA", prefix))
			.unwrap()
			.is_none());

		assert!(storage
			.get_item(&format!("{}/A/.AB.itemdata.json", prefix))
			.unwrap()
			.is_some());
		assert!(storage
			.get_item(&format!("{}/A/AB/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
	}

	#[test]
	fn delete_with_if_match_all() {
		let (storage, prefix, root_etag, A_etag, _, _, AAA_etag, _, _, _) = build_test_db();

		let old_AAA_etag = delete(
			&storage,
			&prefix,
			std::path::Path::new("A/AA/AAA"),
			&crate::Etag::from("*"),
		)
		.unwrap();

		assert_eq!(old_AAA_etag, AAA_etag);

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
		assert!(storage
			.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
			.unwrap()
			.is_none());

		assert!(storage
			.get_item(&format!("{}/A/AA/AAA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert!(storage
			.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert!(storage
			.get_item(&format!("{}/A/AA/AAA/", prefix))
			.unwrap()
			.is_none());
		assert!(storage
			.get_item(&format!("{}/A/AA/AAA", prefix))
			.unwrap()
			.is_none());

		assert!(storage
			.get_item(&format!("{}/A/.AB.itemdata.json", prefix))
			.unwrap()
			.is_some());
		assert!(storage
			.get_item(&format!("{}/A/AB/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
	}

	#[test]
	fn delete_with_existing_folder_conflict() {
		let (storage, prefix, root_etag, A_etag, AA_etag, _, AAA_etag, _, _, _) = build_test_db();

		assert_eq!(
			*delete(
				&storage,
				&prefix,
				std::path::Path::new("A/AA"),
				&crate::Etag::from(""),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::GetError(GetError::Conflict {
				item_path: std::path::PathBuf::from("A/AA/")
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
			.is_some());
		assert_eq!(
			serde_json::from_str::<crate::DataFolder>(
				&storage
					.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			AA_etag
		);

		assert!(storage
			.get_item(&format!("{}/A/AA/AAA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert!(storage
			.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
			.unwrap()
			.is_some());
		assert_eq!(
			serde_json::from_str::<crate::DataDocument>(
				&storage
					.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap()
			.etag,
			AAA_etag
		);
		assert_eq!(
			base64::decode(
				storage
					.get_item(&format!("{}/A/AA/AAA", prefix))
					.unwrap()
					.unwrap()
			)
			.unwrap(),
			b"AAA"
		);

		assert!(storage
			.get_item(&format!("{}/A/.AB.itemdata.json", prefix))
			.unwrap()
			.is_some());
		assert!(storage
			.get_item(&format!("{}/A/AB/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
	}

	#[test]
	fn delete_in_public() {
		let (storage, prefix, root_etag, _, _, _, _, _, _, BA_etag) = build_test_db();

		let old_BA_etag = delete(
			&storage,
			&prefix,
			std::path::Path::new("public/B/BA"),
			&crate::Etag::from(""),
		)
		.unwrap();

		assert_eq!(old_BA_etag, BA_etag);

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
			.get_item(&format!("{}/.public.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert!(storage
			.get_item(&format!("{}/public/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());

		assert!(storage
			.get_item(&format!("{}/public/.B.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert!(storage
			.get_item(&format!("{}/public/B/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());

		assert!(storage
			.get_item(&format!("{}/public/B/.BA.itemdata.json", prefix))
			.unwrap()
			.is_none());
		assert!(storage
			.get_item(&format!("{}/public/B/BA/.folder.itemdata.json", prefix))
			.unwrap()
			.is_none());
	}

	#[test]
	fn delete_in_incorrect_path() {
		let storage = LocalStorageMock::new();
		let prefix = String::from("pontus_onyx_delete_test");
		storage
			.set_item(
				&format!("{}/.folder.itemdata.json", &prefix),
				&serde_json::to_string(&crate::DataFolder::default()).unwrap(),
			)
			.unwrap();

		for i in 0..storage.length().unwrap() {
			dbg!(&storage.key(i).unwrap().unwrap());
		}

		assert_eq!(
			*delete(
				&storage,
				&prefix,
				std::path::Path::new("A/../AA"),
				&crate::Etag::from(""),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::GetError(GetError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/../"),
				error: String::from("`..` is not allowed")
			})
		);

		assert_eq!(storage.length().unwrap(), 1);
		assert_eq!(
			storage.key(0).unwrap().unwrap(),
			format!("{}/.folder.itemdata.json", &prefix)
		);
	}
}
