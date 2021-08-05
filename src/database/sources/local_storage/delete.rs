pub fn delete(
	storage: &dyn super::Storage,
	prefix: &str,
	path: &crate::ItemPath,
	if_match: &crate::Etag,
) -> Result<crate::Etag, Box<dyn std::error::Error>> {
	if path.is_folder() {
		return Err(Box::new(DeleteError::DoesNotWorksForFolders));
	}

	match super::get(storage, prefix, path, if_match, &[], false) {
		Ok(crate::Item::Document { etag: old_etag, .. }) => {
			let parent = path.parent().unwrap_or_else(|| crate::ItemPath::from(""));

			let file_path = parent
				.joined_doc(&format!(".{}.itemdata.json", path.file_name()))
				.unwrap();
			if storage
				.remove_item(&format!("{}/{}", prefix, file_path))
				.is_err()
			{
				return Err(Box::new(DeleteError::CanNotWriteFile {
					item_path: file_path,
					error: String::new(),
				}));
			}

			if storage
				.remove_item(&format!("{}/{}", prefix, file_path))
				.is_err()
			{
				return Err(Box::new(DeleteError::CanNotWriteFile {
					item_path: file_path,
					error: String::new(),
				}));
			}

			if storage
				.remove_item(&format!("{}/{}", prefix, &path))
				.is_err()
			{
				return Err(Box::new(DeleteError::CanNotWriteFile {
					item_path: path.clone(),
					error: String::new(),
				}));
			}

			for parent in path
				.ancestors()
				.into_iter()
				.take(path.ancestors().len().saturating_sub(1))
			{
				let mut has_childs = false;
				for i in 0..storage.length().unwrap_or_default() {
					let key = storage.key(i).unwrap_or_default().unwrap_or_default();
					if key.starts_with(&format!("{}/{}", prefix, parent))
						&& !key.ends_with(".itemdata.json")
					{
						has_childs = true;
						break;
					}
				}
				let mut parent_as_str = format!("{}", parent);
				parent_as_str =
					String::from(parent_as_str.strip_suffix('/').unwrap_or(&parent_as_str));
				if !parent_as_str.is_empty() {
					parent_as_str = format!("{}/", parent_as_str);
				}
				let folder_itemdata_path = crate::ItemPath::from(
					format!("{}/{}.folder.itemdata.json", prefix, parent_as_str).as_str(),
				);

				if !has_childs {
					if storage
						.remove_item(&format!("{}", folder_itemdata_path))
						.is_err()
					{
						return Err(Box::new(DeleteError::CanNotWriteFile {
							item_path: parent,
							error: String::new(),
						}));
					}

				// TODO : eventually cleanup folder content ? (eventually remaining *.itemdata.* files, but should not)
				} else {
					let folder_itemdata_content =
						storage.get_item(&format!("{}", folder_itemdata_path));

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
													&format!("{}", folder_itemdata_path),
													&new_folder_itemdata_content,
												)
												.is_err()
											{
												return Err(Box::new(
													DeleteError::CanNotWriteFile {
														item_path: folder_itemdata_path,
														error: String::new(),
													},
												));
											}
										}
										Err(error) => {
											return Err(Box::new(
												DeleteError::CanNotSerializeFile {
													item_path: folder_itemdata_path,
													error: format!("{}", error),
												},
											));
										}
									}
								}
								Err(error) => {
									return Err(Box::new(DeleteError::CanNotDeserializeFile {
										item_path: folder_itemdata_path,
										error: format!("{}", error),
									}));
								}
							}
						}
						Ok(None) => match serde_json::to_string(&crate::DataFolder::default()) {
							Ok(new_folder_itemdata_content) => {
								if storage
									.set_item(
										&format!("{}", folder_itemdata_path),
										&new_folder_itemdata_content,
									)
									.is_err()
								{
									return Err(Box::new(DeleteError::CanNotWriteFile {
										item_path: folder_itemdata_path,
										error: String::new(),
									}));
								}
							}
							Err(error) => {
								return Err(Box::new(DeleteError::CanNotSerializeFile {
									item_path: folder_itemdata_path,
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
		item_path: crate::ItemPath,
		error: String,
	},
	CanNotReadFile {
		item_path: crate::ItemPath,
		error: String,
	},
	CanNotWriteFile {
		item_path: crate::ItemPath,
		error: String,
	},
	CanNotSerializeFile {
		item_path: crate::ItemPath,
		error: String,
	},
	CanNotDeserializeFile {
		item_path: crate::ItemPath,
		error: String,
	},
}
impl std::fmt::Display for DeleteError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::GetError(get_error) => std::fmt::Display::fmt(get_error, f),
			Self::DoesNotWorksForFolders => f.write_str("this method does not works for folders"),
			Self::CanNotDelete { item_path, error } => f.write_fmt(format_args!(
				"can not delete file `{}` because : {}",
				item_path, error
			)),
			Self::CanNotReadFile { item_path, error } => f.write_fmt(format_args!(
				"can not read file `{}` because : {}",
				item_path, error
			)),
			Self::CanNotWriteFile { item_path, error } => f.write_fmt(format_args!(
				"can not write file `{}` because : {}",
				item_path, error
			)),
			Self::CanNotSerializeFile { item_path, error } => f.write_fmt(format_args!(
				"can not serialize file `{}` because : {}",
				item_path, error
			)),
			Self::CanNotDeserializeFile { item_path, error } => f.write_fmt(format_args!(
				"can not deserialize file `{}` because : {}",
				item_path, error
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
			Self::CanNotDelete {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
			Self::CanNotReadFile {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
			Self::CanNotWriteFile {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
			Self::CanNotSerializeFile {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
			Self::CanNotDeserializeFile {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
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

	use super::{super::GetError, super::LocalStorageMock, super::Storage, delete, DeleteError};
	use crate::{Etag, Item, ItemPath};

	// TODO : test last_modified

	fn build_test_db() -> (
		LocalStorageMock,
		String,
		Etag,
		Etag,
		Etag,
		Etag,
		Etag,
		Etag,
		Etag,
		Etag,
	) {
		let AAA = Item::new_doc(b"AAA", "text/plain");
		let AA = Item::new_folder(vec![("AAA", AAA.clone())]);
		let AB = Item::new_doc(b"AB", "text/plain");
		let A = Item::new_folder(vec![("AA", AA.clone()), ("AB", AB.clone())]);

		let BA = Item::new_doc(b"BA", "text/plain");
		let B = Item::new_folder(vec![("BA", BA.clone())]);
		let public = Item::new_folder(vec![("B", B.clone())]);

		let root = Item::new_folder(vec![("A", A.clone()), ("public", public.clone())]);

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

		if let Item::Document {
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

		if let Item::Document {
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

		if let Item::Document {
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
				&ItemPath::from("A/AA/AAA"),
				&Etag::from(""),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::GetError(GetError::NotFound {
				item_path: ItemPath::from("A/")
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
			&ItemPath::from("A/AA/AAA"),
			&Etag::from(""),
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
			*delete(&storage, &prefix, &ItemPath::from("A/AA/"), &Etag::from(""),)
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
				&ItemPath::from("A/AA/AAA"),
				&Etag::from("OTHER_ETAG"),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::GetError(GetError::NoIfMatch {
				item_path: ItemPath::from("A/AA/AAA"),
				found: AAA_etag.clone(),
				search: Etag::from("OTHER_ETAG")
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

		let old_AAA_etag =
			delete(&storage, &prefix, &ItemPath::from("A/AA/AAA"), &AAA_etag).unwrap();

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
			&ItemPath::from("A/AA/AAA"),
			&Etag::from("*"),
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
			*delete(&storage, &prefix, &ItemPath::from("A/AA"), &Etag::from(""),)
				.unwrap_err()
				.downcast::<DeleteError>()
				.unwrap(),
			DeleteError::GetError(GetError::Conflict {
				item_path: ItemPath::from("A/AA/")
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
			&ItemPath::from("public/B/BA"),
			&Etag::from(""),
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
				&ItemPath::from("A/A\0A"),
				&Etag::from(""),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::GetError(GetError::IncorrectItemName {
				item_path: ItemPath::from("A/A\0A"),
				error: String::from("`A\0A` should not contains `\\0` character")
			})
		);

		assert_eq!(storage.length().unwrap(), 1);
		assert_eq!(
			storage.key(0).unwrap().unwrap(),
			format!("{}/.folder.itemdata.json", &prefix)
		);
	}
}
