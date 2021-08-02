pub fn get(
	storage: &dyn super::Storage,
	prefix: &str,
	path: &std::path::Path,
	if_match: &crate::Etag,
	if_none_match: &[&crate::Etag],
	get_content: bool,
) -> Result<crate::Item, Box<dyn std::error::Error>> {
	if path
		.file_name()
		.unwrap_or_default()
		.to_str()
		.unwrap_or_default()
		.ends_with(".itemdata.json")
	{
		return Err(Box::new(GetError::IsSystemFile));
	}

	if path.to_str().unwrap().starts_with("public/") && path.to_str().unwrap().ends_with('/') {
		return Err(Box::new(GetError::CanNotBeListed {
			item_path: path.to_path_buf(),
		}));
	}

	if path != std::path::PathBuf::from("") {
		let mut cumulated_path = std::path::PathBuf::new();
		let temp_path = path.as_os_str().to_str().unwrap();
		for item_name in temp_path.strip_suffix('/').unwrap_or(temp_path).split('/') {
			cumulated_path.push(item_name);
			if let Err(error) = crate::item_name_is_ok_without_itemdata(item_name) {
				if path != std::path::PathBuf::from("") {
					return Err(Box::new(GetError::IncorrectItemName {
						item_path: cumulated_path,
						error,
					}));
				}
			}
		}
	}

	let folderdata_path = format!(
		"{}/{}.folder.itemdata.json",
		prefix,
		if !path.to_str().unwrap().is_empty() {
			format!(
				"{}/",
				path.to_str()
					.unwrap()
					.strip_suffix('/')
					.unwrap_or_else(|| path.to_str().unwrap())
			)
		} else {
			String::new()
		}
	);

	match storage.get_item(&folderdata_path) {
		Ok(Some(folderdata_content)) => {
			if !path.to_str().unwrap().ends_with('/')
				&& !path.to_str().unwrap().ends_with('\\')
				&& !path.to_string_lossy().is_empty()
			{
				return Err(Box::new(GetError::Conflict {
					item_path: path.to_path_buf(),
				}));
			}

			let content = if get_content {
				let mut content = std::collections::HashMap::new();

				for i in 0..storage.length().unwrap() {
					let key = storage.key(i).unwrap().unwrap();

					if let Some(remain) =
						key.strip_prefix(&format!("{}/{}", prefix, path.to_str().unwrap()))
					{
						if !remain.ends_with(".itemdata.json") {
							if !remain.contains('/') && !remain.contains('\\') {
								content.insert(
									String::from(remain),
									Box::new(
										get(
											storage,
											prefix,
											std::path::Path::new(
												&key.strip_prefix(&format!("{}/", prefix)).unwrap(),
											),
											&crate::Etag::from(""),
											&[],
											true,
										)
										.unwrap(),
									),
								);
							} else {
								let mut ancestors = std::path::Path::new(&remain).ancestors();
								if ancestors.count() > 2 {
									if let Some(name) = ancestors.nth(ancestors.count() - 1 - 1) {
										let name = name.to_str().unwrap();
										if !content.contains_key(name) {
											let target_get = get(
												storage,
												prefix,
												std::path::Path::new(&format!(
													"{}{}/",
													path.to_str().unwrap(),
													&name
												)),
												&crate::Etag::from(""),
												&[],
												true,
											);

											match target_get {
												Ok(item) => {
													content
														.insert(String::from(name), Box::new(item));
												}
												Err(error) => {
													let error =
														*error.downcast::<GetError>().unwrap();
													if let GetError::CanNotBeListed { .. } = error {
														// do nothing (do not add this item)
													} else {
														panic!("{:?}", error);
													}
												}
											}
										}
									}
								}
							}
						}
					}
				}

				Some(content)
			} else {
				None
			};

			match serde_json::from_str::<crate::DataFolder>(&folderdata_content) {
				Ok(folderdata) => {
					if !if_match.is_empty() {
						let upper_if_match = if_match.trim().to_uppercase();
						if folderdata.etag.trim().to_uppercase() != upper_if_match
							&& upper_if_match != "*"
						{
							return Err(Box::new(GetError::NoIfMatch {
								item_path: path.to_path_buf(),
								search: if_match.clone(),
								found: folderdata.etag,
							}));
						}
					}

					if !if_none_match.is_empty() {
						for search_etag in if_none_match {
							if folderdata.etag.trim().to_uppercase()
								== search_etag.trim().to_uppercase()
								|| search_etag.trim() == "*"
							{
								return Err(Box::new(GetError::IfNoneMatch {
									item_path: path.to_path_buf(),
									search: (*search_etag).clone(),
									found: folderdata.etag,
								}));
							}
						}
					}

					return Ok(crate::Item::Folder {
						etag: folderdata.etag,
						content,
					});
				}
				Err(error) => {
					return Err(Box::new(GetError::CanNotSerializeFile {
						item_path: std::path::PathBuf::from(folderdata_path),
						error: format!("{}", error),
					}))
				}
			}
		}
		Ok(None) => {
			let target_parent = path.parent().unwrap_or_else(|| std::path::Path::new(""));
			let filedata_path = format!(
				"{}/{}{}",
				prefix,
				if target_parent != std::path::Path::new("") {
					format!("{}/", target_parent.to_str().unwrap())
				} else {
					String::new()
				},
				format!(
					".{}.itemdata.json",
					path.file_name().unwrap_or_default().to_str().unwrap()
				)
			);

			match storage.get_item(&filedata_path) {
				Ok(Some(filedata_content)) => {
					if path.to_str().unwrap().ends_with('/')
						|| path.to_str().unwrap().ends_with('\\')
					{
						return Err(Box::new(GetError::Conflict {
							item_path: path.to_path_buf(),
						}));
					}

					match serde_json::from_str::<crate::DataDocument>(&filedata_content) {
						Ok(filedata) => {
							let content = if get_content {
								match storage.get_item(&format!(
									"{}/{}",
									prefix,
									path.to_str().unwrap()
								)) {
									Ok(Some(content)) => match base64::decode(content) {
										Ok(content) => Some(content),
										Err(_) => None,
									},
									_ => None,
								}
							} else {
								None
							};

							if !if_match.is_empty() {
								let upper_if_match = if_match.trim().to_uppercase();
								if filedata.etag.trim().to_uppercase() != upper_if_match
									&& upper_if_match != "*"
								{
									return Err(Box::new(GetError::NoIfMatch {
										item_path: path.to_path_buf(),
										search: if_match.clone(),
										found: filedata.etag,
									}));
								}
							}

							if !if_none_match.is_empty() {
								for search_etag in if_none_match {
									if filedata.etag.trim().to_uppercase()
										== search_etag.trim().to_uppercase() || search_etag.trim()
										== "*"
									{
										return Err(Box::new(GetError::IfNoneMatch {
											item_path: path.to_path_buf(),
											search: (*search_etag).clone(),
											found: filedata.etag,
										}));
									}
								}
							}

							return Ok(crate::Item::Document {
								etag: filedata.etag,
								content_type: filedata.content_type,
								last_modified: filedata.last_modified,
								content,
							});
						}
						Err(error) => {
							return Err(Box::new(GetError::CanNotSerializeFile {
								item_path: std::path::PathBuf::from(filedata_path),
								error: format!("{}", error),
							}))
						}
					}
				}
				Ok(None) => {
					let filedata_path = format!(
						"{}/{}.folder.itemdata.json",
						prefix,
						if !path.to_str().unwrap().is_empty() {
							format!(
								"{}/",
								path.to_str()
									.unwrap()
									.strip_suffix('/')
									.unwrap_or_else(|| path.to_str().unwrap())
							)
						} else {
							String::new()
						}
					);
					match storage.get_item(&filedata_path) {
						Ok(Some(_)) => {
							return Err(Box::new(GetError::Conflict {
								item_path: std::path::PathBuf::from(
									path.to_str()
										.unwrap()
										.strip_suffix('/')
										.unwrap_or_else(|| path.to_str().unwrap()),
								),
							}));
						}
						Ok(None) => {
							if path != std::path::Path::new("") {
								let parent =
									path.parent().unwrap_or_else(|| std::path::Path::new(""));

								let parent_get = get(
									storage,
									prefix,
									parent,
									&crate::Etag::from(""),
									&[],
									false,
								);

								if let Ok(crate::Item::Document { .. }) = parent_get {
									return Err(Box::new(GetError::Conflict {
										item_path: parent.to_path_buf(),
									}));
								}

								if let Err(error) = parent_get {
									if let GetError::NotFound { item_path } =
										*error.downcast::<GetError>().unwrap()
									{
										return Err(Box::new(GetError::NotFound { item_path }));
									}
								}
							}

							return Err(Box::new(GetError::NotFound {
								item_path: path.to_path_buf(),
							}));
						}
						Err(_) => return Err(Box::new(GetError::CanNotGetStorage)),
					}
				}
				Err(_) => return Err(Box::new(GetError::CanNotGetStorage)),
			}
		}
		Err(_) => return Err(Box::new(GetError::CanNotGetStorage)),
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum GetError {
	Conflict {
		item_path: std::path::PathBuf,
	},
	NotFound {
		item_path: std::path::PathBuf,
	},
	IncorrectItemName {
		item_path: std::path::PathBuf,
		error: String,
	},
	CanNotBeListed {
		item_path: std::path::PathBuf,
	},
	NoIfMatch {
		item_path: std::path::PathBuf,
		search: crate::Etag,
		found: crate::Etag,
	},
	IfNoneMatch {
		item_path: std::path::PathBuf,
		search: crate::Etag,
		found: crate::Etag,
	},
	CanNotGetStorage,
	CanNotSerializeFile {
		item_path: std::path::PathBuf,
		error: String,
	},
	IsSystemFile,
}
impl std::fmt::Display for GetError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Conflict { item_path } => f.write_fmt(format_args!("name conflict between folder and file on the path `{}`", item_path.to_string_lossy())),
			Self::NotFound { item_path } => f.write_fmt(format_args!("path not found : `{}`", item_path.to_string_lossy())),
			Self::IncorrectItemName { item_path, error } => f.write_fmt(format_args!("the path `{}` is incorrect, because {}", item_path.to_string_lossy(), error)),
			Self::CanNotBeListed { item_path } => f.write_fmt(format_args!("the folder `{}` can not be listed", item_path.to_string_lossy())),
			Self::NoIfMatch { item_path, search, found } => f.write_fmt(format_args!("the requested `{}` etag (through `IfMatch`) for `{}` was not found, found `{}` instead", search, item_path.to_string_lossy(), found)),
			Self::IfNoneMatch { item_path, search, found } => f.write_fmt(format_args!("the unwanted etag `{}` (through `IfNoneMatch`) for `{}` was matches with `{}`", search, item_path.to_string_lossy(), found)),
			Self::CanNotGetStorage => f.write_str("can not get storage"),
			Self::CanNotSerializeFile { item_path, error } => f.write_fmt(format_args!("can not parse file `{}` because {}", item_path.to_string_lossy(), error)),
			Self::IsSystemFile => f.write_str("this is a system file, that should not be server"),
		}
	}
}
impl std::error::Error for GetError {}
#[cfg(feature = "server_bin")]
impl crate::database::Error for GetError {
	fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse {
		match self {
			Self::Conflict { item_path } => {
				if item_path.starts_with("public/") {
					crate::database::build_http_json_response(
						origin,
						&actix_web::http::Method::GET,
						actix_web::http::StatusCode::NOT_FOUND,
						None,
						Some(format!(
							"path not found : `{}`",
							item_path.to_string_lossy()
						)),
						should_have_body,
					)
				} else {
					crate::database::build_http_json_response(
						origin,
						&actix_web::http::Method::GET,
						actix_web::http::StatusCode::CONFLICT,
						None,
						Some(format!("{}", self)),
						should_have_body,
					)
				}
			}
			Self::NotFound { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::IncorrectItemName { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::CanNotBeListed { item_path } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!(
					"path not found : `{}`",
					item_path.to_string_lossy()
				)),
				should_have_body,
			),
			Self::NoIfMatch { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::IfNoneMatch { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::CanNotGetStorage => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::CanNotSerializeFile { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
			Self::IsSystemFile => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::BAD_REQUEST,
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

	use super::{super::LocalStorageMock, super::Storage, get, GetError};

	// TODO : test if folderdata found but content is file
	// TODO : test if filedata found but content is folder

	#[test]
	fn all_tests_bulk() {
		let AA = crate::Item::new_doc(b"AA", "text/plain");
		let AB = crate::Item::new_doc(b"AB", "text/plain");
		let AC = crate::Item::new_doc(b"AC", "text/plain");
		let BA = crate::Item::new_doc(b"BA", "text/plain");
		let BB = crate::Item::new_doc(b"BB", "text/plain");
		let CA = crate::Item::new_doc(b"CA", "text/plain");

		let A = crate::Item::new_folder(vec![
			("AA", AA.clone()),
			("AB", AB.clone()),
			("AC", AC.clone()),
		]);
		let B = crate::Item::new_folder(vec![("BA", BA.clone()), ("BB", BB.clone())]);
		let C = crate::Item::new_folder(vec![("CA", CA.clone())]);
		let public = crate::Item::new_folder(vec![("C", C.clone())]);

		let root = crate::Item::new_folder(vec![
			("A", A.clone()),
			("B", B.clone()),
			("public", public.clone()),
		]);

		let mut root_without_public = root.clone();
		if let crate::Item::Folder {
			content: Some(content),
			..
		} = &mut root_without_public
		{
			content.remove("public").unwrap();
		} else {
			panic!()
		}

		////////////////////////////////////////////////////////////////////////////////////////////////

		let prefix = "pontus_onyx_get_test";

		let storage = LocalStorageMock::new();

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
		}

		if let crate::Item::Document {
			content: Some(content),
			etag: AC_etag,
			content_type: AC_content_type,
			last_modified: AC_last_modified,
		} = AC.clone()
		{
			storage
				.set_item(
					&format!("{}/A/.AC.itemdata.json", prefix),
					&serde_json::to_string(&crate::DataDocument {
						datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
						etag: AC_etag,
						content_type: AC_content_type,
						last_modified: AC_last_modified,
					})
					.unwrap(),
				)
				.unwrap();

			storage
				.set_item(&format!("{}/A/AC", prefix), &base64::encode(&content))
				.unwrap();
		}

		storage
			.set_item(
				&format!("{}/B/.folder.itemdata.json", prefix),
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
					&format!("{}/B/.BA.itemdata.json", prefix),
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
				.set_item(&format!("{}/B/BA", prefix), &base64::encode(&content))
				.unwrap();
		}

		if let crate::Item::Document {
			content: Some(content),
			etag: BB_etag,
			content_type: BB_content_type,
			last_modified: BB_last_modified,
		} = BB.clone()
		{
			storage
				.set_item(
					&format!("{}/B/.BB.itemdata.json", prefix),
					&serde_json::to_string(&crate::DataDocument {
						datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
						etag: BB_etag,
						content_type: BB_content_type,
						last_modified: BB_last_modified,
					})
					.unwrap(),
				)
				.unwrap();

			storage
				.set_item(&format!("{}/B/BB", prefix), &base64::encode(&content))
				.unwrap();
		}

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

		storage
			.set_item(
				&format!("{}/public/C/.folder.itemdata.json", prefix),
				&serde_json::to_string(&crate::DataFolder {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: C.get_etag().clone(),
				})
				.unwrap(),
			)
			.unwrap();

		if let crate::Item::Document {
			content: Some(content),
			etag: CA_etag,
			content_type: CA_content_type,
			last_modified: CA_last_modified,
		} = CA.clone()
		{
			storage
				.set_item(
					&format!("{}/public/C/.CA.itemdata.json", prefix),
					&serde_json::to_string(&crate::DataDocument {
						datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
						etag: CA_etag,
						content_type: CA_content_type,
						last_modified: CA_last_modified,
					})
					.unwrap(),
				)
				.unwrap();

			storage
				.set_item(
					&format!("{}/public/C/CA", prefix),
					&base64::encode(&content),
				)
				.unwrap();
		}

		let mut keys = vec![];
		for id in 0..storage.length().unwrap() {
			let key = storage.key(id).unwrap().unwrap();
			keys.push(key);
		}
		keys.sort();
		dbg!(keys);

		////////////////////////////////////////////////////////////////////////////////////////////////

		println!("//////// 010 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new(""),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap(),
			root_without_public.clone()
		);
		println!("//////// 020 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new("A/"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap(),
			A.clone()
		);
		println!("//////// 030 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new("A/AA"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap(),
			AA.clone()
		);
		println!("//////// 040 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new("A/AB"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap(),
			AB
		);
		println!("//////// 050 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new("A/AC"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap(),
			AC
		);
		println!("//////// 060 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new("B/"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap(),
			B
		);
		println!("//////// 070 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new("B/BA"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap(),
			BA
		);
		println!("//////// 080 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new("B/BB"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap(),
			BB
		);
		println!("//////// 090 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new("public/C/CA"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap(),
			CA
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		println!("//////// 100 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new(""),
				root.get_etag(),
				&vec![],
				true
			)
			.unwrap(),
			root_without_public.clone()
		);
		println!("//////// 110 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new("A/"),
				A.get_etag(),
				&vec![],
				true
			)
			.unwrap(),
			A.clone()
		);
		println!("//////// 120 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new("A/AA"),
				AA.get_etag(),
				&vec![],
				true
			)
			.unwrap(),
			AA.clone()
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		println!("//////// 130 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new(""),
				&crate::Etag::from(""),
				&[&crate::Etag::from("ANOTHER_ETAG")],
				true
			)
			.unwrap(),
			root_without_public.clone()
		);
		println!("//////// 140 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new("A/"),
				&crate::Etag::from(""),
				&[&crate::Etag::from("ANOTHER_ETAG")],
				true
			)
			.unwrap(),
			A.clone()
		);
		println!("//////// 150 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new("A/AA"),
				&crate::Etag::from(""),
				&[&crate::Etag::from("ANOTHER_ETAG")],
				true
			)
			.unwrap(),
			AA.clone()
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		println!("//////// 160 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::Conflict {
				item_path: std::path::PathBuf::from("A")
			}
		);
		println!("//////// 170 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A/AA/"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::Conflict {
				item_path: std::path::PathBuf::from("A/AA/")
			}
		);
		println!("//////// 180 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A/AC/not_exists"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::Conflict {
				item_path: std::path::PathBuf::from("A/AC/")
			}
		);
		println!("//////// 190 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A/not_exists"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NotFound {
				item_path: std::path::PathBuf::from("A/not_exists")
			}
		);
		println!("//////// 200 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A/not_exists/nested"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NotFound {
				item_path: std::path::PathBuf::from("A/not_exists/")
			}
		);
		println!("//////// 210 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("B/not_exists"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NotFound {
				item_path: std::path::PathBuf::from("B/not_exists")
			}
		);
		println!("//////// 220 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("not_exists/"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NotFound {
				item_path: std::path::PathBuf::from("not_exists/")
			}
		);
		println!("//////// 230 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("not_exists"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NotFound {
				item_path: std::path::PathBuf::from("not_exists")
			}
		);
		println!("//////// 240 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("."),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IncorrectItemName {
				item_path: std::path::PathBuf::from("."),
				error: String::from("`.` is not allowed")
			}
		);
		println!("//////// 250 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A/.."),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/.."),
				error: String::from("`..` is not allowed")
			}
		);
		println!("//////// 260 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A/../"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/../"),
				error: String::from("`..` is not allowed")
			}
		);
		println!("//////// 270 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A/../AA"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/../"),
				error: String::from("`..` is not allowed")
			}
		);
		println!("//////// 280 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A/A\0A"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/A\0A"),
				error: format!("`{}` should not contains `\\0` character", "A\0A")
			}
		);
		println!("//////// 290 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("public/"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::CanNotBeListed {
				item_path: std::path::PathBuf::from("public/")
			},
		);
		println!("//////// 300 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("public/C/"),
				&crate::Etag::from(""),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::CanNotBeListed {
				item_path: std::path::PathBuf::from("public/C/")
			}
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		println!("//////// 310 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new(""),
				&crate::Etag::from("ANOTHER_ETAG"),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NoIfMatch {
				item_path: std::path::PathBuf::from(""),
				search: crate::Etag::from("ANOTHER_ETAG"),
				found: root.get_etag().clone()
			}
		);
		println!("//////// 320 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A/"),
				&crate::Etag::from("ANOTHER_ETAG"),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NoIfMatch {
				item_path: std::path::PathBuf::from("A/"),
				search: crate::Etag::from("ANOTHER_ETAG"),
				found: A.get_etag().clone()
			}
		);
		println!("//////// 330 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A/AA"),
				&crate::Etag::from("ANOTHER_ETAG"),
				&vec![],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NoIfMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				search: crate::Etag::from("ANOTHER_ETAG"),
				found: AA.get_etag().clone()
			}
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		println!("//////// 340 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new(""),
				&crate::Etag::from(""),
				&[&crate::Etag::from("*")],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IfNoneMatch {
				item_path: std::path::PathBuf::from(""),
				search: crate::Etag::from("*"),
				found: root.get_etag().clone()
			}
		);
		println!("//////// 350 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A/"),
				&crate::Etag::from(""),
				&[&crate::Etag::from("*")],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IfNoneMatch {
				item_path: std::path::PathBuf::from("A/"),
				search: crate::Etag::from("*"),
				found: A.get_etag().clone()
			}
		);
		println!("//////// 360 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A/AA"),
				&crate::Etag::from(""),
				&[&crate::Etag::from("*")],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IfNoneMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				search: crate::Etag::from("*"),
				found: AA.get_etag().clone()
			}
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		println!("//////// 370 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new(""),
				&crate::Etag::from(""),
				&[root.get_etag()],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IfNoneMatch {
				item_path: std::path::PathBuf::from(""),
				search: root.get_etag().clone(),
				found: root.get_etag().clone()
			}
		);
		println!("//////// 380 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A/"),
				&crate::Etag::from(""),
				&[A.get_etag()],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IfNoneMatch {
				item_path: std::path::PathBuf::from("A/"),
				search: A.get_etag().clone(),
				found: A.get_etag().clone()
			}
		);
		println!("//////// 390 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A/AA"),
				&crate::Etag::from(""),
				&[AA.get_etag()],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IfNoneMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				search: AA.get_etag().clone(),
				found: AA.get_etag().clone()
			}
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		println!("//////// 400 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new(""),
				&crate::Etag::from(""),
				&[],
				false
			)
			.unwrap(),
			root_without_public.empty_clone()
		);
		println!("//////// 410 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new("A/"),
				&crate::Etag::from(""),
				&[],
				false
			)
			.unwrap(),
			A.empty_clone()
		);
		println!("//////// 420 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new("A/AA"),
				&crate::Etag::from(""),
				&[],
				false
			)
			.unwrap(),
			AA.empty_clone()
		);
		println!("//////// 430 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("public/"),
				&crate::Etag::from(""),
				&[],
				false
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::CanNotBeListed {
				item_path: std::path::PathBuf::from("public/")
			}
		);
		println!("//////// 440 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("public/C/"),
				&crate::Etag::from(""),
				&[],
				false
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::CanNotBeListed {
				item_path: std::path::PathBuf::from("public/C/")
			}
		);
		println!("//////// 450 ////////");
		assert_eq!(
			get(
				&storage,
				prefix,
				std::path::Path::new("public/C/CA"),
				&crate::Etag::from(""),
				&[],
				false
			)
			.unwrap(),
			CA.empty_clone()
		);
		println!("//////// 460 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("public/not_exists"),
				&crate::Etag::from(""),
				&[],
				false
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NotFound {
				item_path: std::path::PathBuf::from("public/not_exists")
			}
		);
		println!("//////// 470 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("public/not_exists/"),
				&crate::Etag::from(""),
				&[],
				false
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::CanNotBeListed {
				item_path: std::path::PathBuf::from("public/not_exists/")
			}
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		println!("//////// 480 ////////");
		assert_eq!(
			*get(
				&storage,
				prefix,
				std::path::Path::new("A/.AA.itemdata.json"),
				&crate::Etag::from(""),
				&[&crate::Etag::from("*")],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IsSystemFile
		);
	}
}
