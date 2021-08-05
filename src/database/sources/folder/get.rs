pub fn get(
	root_folder_path: &std::path::Path,
	path: &crate::ItemPath,
	if_match: &crate::Etag,
	if_none_match: &[&crate::Etag],
	get_content: bool,
) -> Result<crate::Item, Box<dyn std::error::Error>> {
	if path.ends_with(".itemdata.toml") {
		return Err(Box::new(GetError::IsSystemFile));
	}

	if path.starts_with("public/") && path.is_folder() {
		return Err(Box::new(GetError::CanNotBeListed {
			item_path: path.clone(),
		}));
	}

	if path != &crate::ItemPath::from("") {
		let mut cumulated_path = crate::ItemPath::from("");
		for part in path.parts_iter() {
			cumulated_path = cumulated_path.joined(part).unwrap();
			if let Err(error) = crate::item_name_is_ok_without_itemdata(part.name()) {
				return Err(Box::new(GetError::IncorrectItemName {
					item_path: cumulated_path,
					error,
				}));
			}
		}
	}

	let target = root_folder_path.join(std::path::PathBuf::from(path));
	// need to cast `path` into `&str` because `PathBuf::from("A/").ends_with("/") == false` !
	if !(target.to_str().unwrap().ends_with('/') || target.to_str().unwrap().ends_with('\\'))
		&& target.is_file()
	{
		if target.exists() {
			let itemdata_file_path = target.parent().unwrap().join(format!(
				".{}.itemdata.toml",
				target.file_name().unwrap().to_os_string().to_str().unwrap()
			));

			match std::fs::read(&itemdata_file_path) {
				Ok(itemdata_file_content) => {
					match toml::from_slice::<crate::DataDocument>(&itemdata_file_content) {
						Ok(itemdata) => {
							if !if_match.is_empty() && &itemdata.etag != if_match && if_match != "*"
							{
								return Err(Box::new(GetError::NoIfMatch {
									item_path: path.clone(),
									search: if_match.clone(),
									found: itemdata.etag,
								}));
							}

							if !if_none_match.is_empty() {
								for none_match in if_none_match {
									if &&itemdata.etag == none_match || *none_match == "*" {
										return Err(Box::new(GetError::IfNoneMatch {
											item_path: path.clone(),
											search: (*none_match).clone(),
											found: itemdata.etag,
										}));
									}
								}
							}

							if get_content {
								match std::fs::read(&target) {
									Ok(file_content) => {
										return Ok(crate::Item::Document {
											content: Some(file_content),
											content_type: itemdata.content_type,
											etag: itemdata.etag,
											last_modified: itemdata.last_modified,
										});
									}
									Err(error) => {
										return Err(Box::new(GetError::CanNotReadFile {
											os_path: target,
											error: format!("{}", error),
										}));
									}
								}
							} else {
								return Ok(crate::Item::Document {
									content: None,
									content_type: itemdata.content_type,
									etag: itemdata.etag,
									last_modified: itemdata.last_modified,
								});
							}
						}
						Err(error) => {
							return Err(Box::new(GetError::CanNotDeserializeFile {
								os_path: itemdata_file_path,
								error: format!("{}", error),
							}));
						}
					}
				}
				Err(error) => {
					return Err(Box::new(GetError::CanNotReadFile {
						os_path: itemdata_file_path,
						error: format!("{}", error),
					}));
				}
			}
		} else {
			return Err(Box::new(GetError::NotFound {
				item_path: path.clone(),
			}));
		}
	// need to cast `path` into `&str` because `PathBuf::from("A/").ends_with("/") == false` !
	} else if (target.to_str().unwrap().ends_with('/')
		|| target.to_str().unwrap().ends_with('\\')
		|| target == std::path::PathBuf::from(""))
		&& target.is_dir()
	{
		if target.exists() {
			let itemdata_file_path = target.join(".folder.itemdata.toml");

			match std::fs::read(&itemdata_file_path) {
				Ok(itemdata_file_content) => {
					match toml::from_slice::<crate::DataFolder>(&itemdata_file_content) {
						Ok(itemdata) => {
							if !if_match.is_empty() && &itemdata.etag != if_match && if_match != "*"
							{
								return Err(Box::new(GetError::NoIfMatch {
									item_path: path.clone(),
									search: if_match.clone(),
									found: itemdata.etag,
								}));
							}

							if !if_none_match.is_empty() {
								for none_match in if_none_match {
									if &&itemdata.etag == none_match || *none_match == "*" {
										return Err(Box::new(GetError::IfNoneMatch {
											item_path: path.clone(),
											search: (*none_match).clone(),
											found: itemdata.etag,
										}));
									}
								}
							}

							if get_content {
								if !path.starts_with("public") {
									match std::fs::read_dir(&target) {
										Ok(dir_contents) => {
											let mut dir_items = std::collections::HashMap::new();
											for dir_content in dir_contents {
												match dir_content {
													Ok(dir_entry) => {
														let entry_name = String::from(
															dir_entry.file_name().to_str().unwrap(),
														);
														if !entry_name.ends_with(".itemdata.toml") {
															let entry_item =
																get(
																	root_folder_path,
																	&path
																		.joined(&if dir_entry
																			.file_type()
																			.unwrap()
																			.is_dir()
																		{
																			crate::ItemPathPart::Folder(entry_name.clone())
																		} else {
																			crate::ItemPathPart::Document(entry_name.clone())
																		})
																		.unwrap(),
																	&crate::Etag::from(""),
																	&[],
																	get_content,
																);

															match entry_item {
																Ok(entry_item) => {
																	dir_items.insert(
																		entry_name,
																		Box::new(entry_item),
																	);
																}
																Err(error) => {
																	if let Some(
																		GetError::CanNotBeListed {
																			..
																		},
																	) = error
																		.downcast_ref::<GetError>()
																	{
																		// do nothing (do not add this item)
																	} else {
																		return Err(error);
																	}
																}
															}
														}
													}
													Err(error) => {
														return Err(Box::new(GetError::IOError {
															error: format!("{}", error),
														}));
													}
												}
											}

											return Ok(crate::Item::Folder {
												etag: itemdata.etag,
												content: Some(dir_items),
											});
										}
										Err(error) => {
											return Err(Box::new(GetError::CanNotReadFile {
												os_path: target,
												error: format!("{}", error),
											}));
										}
									}
								} else {
									return Err(Box::new(GetError::CanNotBeListed {
										item_path: path.clone(),
									}));
								}
							} else {
								return Ok(crate::Item::Folder {
									etag: itemdata.etag,
									content: None,
								});
							}
						}
						Err(error) => {
							return Err(Box::new(GetError::CanNotDeserializeFile {
								os_path: itemdata_file_path,
								error: format!("{}", error),
							}));
						}
					}
				}
				Err(error) => {
					return Err(Box::new(GetError::CanNotReadFile {
						os_path: itemdata_file_path,
						error: format!("{}", error),
					}));
				}
			}
		} else {
			return Err(Box::new(GetError::NotFound {
				item_path: path.clone(),
			}));
		}
	} else if target.is_file()
		|| target.is_dir()
		|| std::path::PathBuf::from(
			target
				.to_str()
				.unwrap()
				.strip_suffix('/')
				.unwrap_or_else(|| target.to_str().unwrap()),
		)
		.is_file()
	{
		return Err(Box::new(GetError::Conflict {
			item_path: path.folder_clone(),
		}));
	} else if let Some(parent) = path.parent() {
		let get_parent = get(
			root_folder_path,
			&parent,
			&crate::Etag::from(""),
			&[],
			false,
		);
		if let Err(get_parent) = get_parent {
			let get_parent: GetError = *get_parent.downcast().unwrap();
			if let GetError::Conflict { item_path: _ } = &get_parent {
				return Err(Box::new(get_parent));
			} else if let GetError::NotFound { item_path: _ } = &get_parent {
				return Err(Box::new(get_parent));
			} else {
				return Err(Box::new(GetError::NotFound {
					item_path: path.clone(),
				}));
			}
		} else if root_folder_path
			.join(std::path::PathBuf::from(&path.document_clone()))
			.exists()
		{
			return Err(Box::new(GetError::Conflict {
				item_path: path.document_clone(),
			}));
		} else {
			return Err(Box::new(GetError::NotFound {
				item_path: path.clone(),
			}));
		}
	} else {
		return Err(Box::new(GetError::NotFound {
			item_path: path.clone(),
		}));
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum GetError {
	Conflict {
		item_path: crate::ItemPath,
	},
	NotFound {
		item_path: crate::ItemPath,
	},
	IncorrectItemName {
		item_path: crate::ItemPath,
		error: String,
	},
	CanNotBeListed {
		item_path: crate::ItemPath,
	},
	NoIfMatch {
		item_path: crate::ItemPath,
		search: crate::Etag,
		found: crate::Etag,
	},
	IfNoneMatch {
		item_path: crate::ItemPath,
		search: crate::Etag,
		found: crate::Etag,
	},
	CanNotReadFile {
		os_path: std::path::PathBuf,
		error: String,
	},
	CanNotDeserializeFile {
		os_path: std::path::PathBuf,
		error: String,
	},
	IOError {
		error: String,
	},
	IsSystemFile,
}
impl std::fmt::Display for GetError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Conflict { item_path } => f.write_fmt(format_args!("name conflict between folder and file on the path `{}`", item_path)),
			Self::NotFound { item_path } => f.write_fmt(format_args!("path not found : `{}`", item_path)),
			Self::IncorrectItemName { item_path, error } => f.write_fmt(format_args!("the path `{}` is incorrect, because {}", item_path, error)),
			Self::CanNotBeListed { item_path } => f.write_fmt(format_args!("the folder `{}` can not be listed", item_path)),
			Self::NoIfMatch { item_path, search, found } => f.write_fmt(format_args!("the requested `{}` etag (through `IfMatch`) for `{}` was not found, found `{}` instead", search, item_path, found)),
			Self::IfNoneMatch { item_path, search, found } => f.write_fmt(format_args!("the unwanted etag `{}` (through `IfNoneMatch`) for `{}` was matches with `{}`", search, item_path, found)),
			Self::CanNotReadFile { os_path, error } => f.write_fmt(format_args!("can not read file `{:?}`, because {}", os_path, error)),
			Self::CanNotDeserializeFile { os_path, error } => f.write_fmt(format_args!("can not deserialize file `{:?}`, because {}", os_path, error)),
			Self::IOError { error } => f.write_fmt(format_args!("file system error : {}", error)),
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
						Some(format!("path not found : `{}`", item_path)),
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
			Self::NotFound { item_path: _ } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::IncorrectItemName {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
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
				Some(format!("path not found : `{}`", item_path)),
				should_have_body,
			),
			Self::NoIfMatch {
				item_path: _,
				search: _,
				found: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::IfNoneMatch {
				item_path: _,
				search: _,
				found: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::CanNotReadFile {
				os_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
			Self::CanNotDeserializeFile {
				os_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
			Self::IOError { error: _ } => crate::database::build_http_json_response(
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

	use super::{get, GetError};
	use crate::{Etag, Item, ItemPath};
	use std::convert::TryFrom;

	#[test]
	fn all_tests_bulk() {
		let AA = Item::new_doc(b"AA", "text/plain");
		let AB = Item::new_doc(b"AB", "text/plain");
		let AC = Item::new_doc(b"AC", "text/plain");
		let BA = Item::new_doc(b"BA", "text/plain");
		let BB = Item::new_doc(b"BB", "text/plain");
		let CA = Item::new_doc(b"CA", "text/plain");

		let A = Item::new_folder(vec![
			("AA", AA.clone()),
			("AB", AB.clone()),
			("AC", AC.clone()),
		]);
		let B = Item::new_folder(vec![("BA", BA.clone()), ("BB", BB.clone())]);
		let C = Item::new_folder(vec![("CA", CA.clone())]);
		let public = Item::new_folder(vec![("C", C.clone())]);

		let root = Item::new_folder(vec![
			("A", A.clone()),
			("B", B.clone()),
			("public", public.clone()),
		]);

		let mut root_without_public = root.clone();
		if let Item::Folder {
			content: Some(content),
			..
		} = &mut root_without_public
		{
			content.remove("public").unwrap();
		} else {
			panic!()
		}

		////////////////////////////////////////////////////////////////////////////////////////////////

		let tmp_folder = tempfile::tempdir().unwrap();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		let root_path = tmp_folder_path.clone();

		let A_path = tmp_folder_path.join("A");
		let B_path = tmp_folder_path.join("B");
		let public_path = tmp_folder_path.join("public");
		let C_path = public_path.join("C");
		let AA_path = A_path.join("AA");
		let AB_path = A_path.join("AB");
		let AC_path = A_path.join("AC");
		let BA_path = B_path.join("BA");
		let BB_path = B_path.join("BB");
		let CA_path = C_path.join("CA");

		std::fs::create_dir_all(&A_path).unwrap();
		std::fs::create_dir_all(&B_path).unwrap();
		std::fs::create_dir_all(&C_path).unwrap();

		let root_data_path = tmp_folder_path.join(".folder.itemdata.toml");
		std::fs::write(
			&root_data_path,
			toml::to_string(&crate::DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: root.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

		let A_data_path = A_path.join(".folder.itemdata.toml");
		std::fs::write(
			A_data_path,
			toml::to_string(&crate::DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: A.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

		let B_data_path = B_path.join(".folder.itemdata.toml");
		std::fs::write(
			B_data_path,
			toml::to_string(&crate::DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: B.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

		let public_data_path = public_path.join(".folder.itemdata.toml");
		std::fs::write(
			public_data_path,
			toml::to_string(&crate::DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: public.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

		let C_data_path = C_path.join(".folder.itemdata.toml");
		std::fs::write(
			C_data_path,
			toml::to_string(&crate::DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: C.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

		let AA_data_path = A_path.join(".AA.itemdata.toml");
		std::fs::write(
			AA_data_path,
			toml::to_string(&crate::DataDocument::try_from(AA.clone()).unwrap()).unwrap(),
		)
		.unwrap();

		let AB_data_path = A_path.join(".AB.itemdata.toml");
		std::fs::write(
			AB_data_path,
			toml::to_string(&crate::DataDocument::try_from(AB.clone()).unwrap()).unwrap(),
		)
		.unwrap();

		let AC_data_path = A_path.join(".AC.itemdata.toml");
		std::fs::write(
			AC_data_path,
			toml::to_string(&crate::DataDocument::try_from(AC.clone()).unwrap()).unwrap(),
		)
		.unwrap();

		let BA_data_path = B_path.join(".BA.itemdata.toml");
		std::fs::write(
			BA_data_path,
			toml::to_string(&crate::DataDocument::try_from(BA.clone()).unwrap()).unwrap(),
		)
		.unwrap();

		let BB_data_path = B_path.join(".BB.itemdata.toml");
		std::fs::write(
			BB_data_path,
			toml::to_string(&&crate::DataDocument::try_from(BB.clone()).unwrap()).unwrap(),
		)
		.unwrap();

		let CA_data_path = C_path.join(".CA.itemdata.toml");
		std::fs::write(
			CA_data_path,
			toml::to_string(&&crate::DataDocument::try_from(CA.clone()).unwrap()).unwrap(),
		)
		.unwrap();

		if let Item::Document {
			content: Some(content),
			..
		} = &AA
		{
			std::fs::write(AA_path, content).unwrap();
		} else {
			panic!()
		}

		if let Item::Document {
			content: Some(content),
			..
		} = &AB
		{
			std::fs::write(AB_path, content).unwrap();
		} else {
			panic!()
		}

		if let Item::Document {
			content: Some(content),
			..
		} = &AC
		{
			std::fs::write(AC_path, content).unwrap();
		} else {
			panic!()
		}

		if let Item::Document {
			content: Some(content),
			..
		} = &BA
		{
			std::fs::write(BA_path, content).unwrap();
		} else {
			panic!()
		}

		if let Item::Document {
			content: Some(content),
			..
		} = &BB
		{
			std::fs::write(BB_path, content).unwrap();
		} else {
			panic!()
		}

		if let Item::Document {
			content: Some(content),
			..
		} = &CA
		{
			std::fs::write(CA_path, content).unwrap();
		} else {
			panic!()
		}

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			get(&root_path, &ItemPath::from(""), &Etag::from(""), &[], true).unwrap(),
			root_without_public.clone()
		);
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("A/"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap(),
			A.clone()
		);
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("A/AA"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap(),
			AA.clone()
		);
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("A/AB"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap(),
			AB
		);
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("A/AC"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap(),
			AC
		);
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("B/"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap(),
			B
		);
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("B/BA"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap(),
			BA
		);
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("B/BB"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap(),
			BB
		);
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("public/C/CA"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap(),
			CA
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			get(&root_path, &ItemPath::from(""), root.get_etag(), &[], true).unwrap(),
			root_without_public.clone()
		);
		assert_eq!(
			get(&root_path, &ItemPath::from("A/"), A.get_etag(), &[], true).unwrap(),
			A.clone()
		);
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("A/AA"),
				AA.get_etag(),
				&[],
				true
			)
			.unwrap(),
			AA.clone()
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			get(
				&root_path,
				&ItemPath::from(""),
				&Etag::from(""),
				&[&Etag::from("ANOTHER_ETAG")],
				true
			)
			.unwrap(),
			root_without_public.clone()
		);
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("A/"),
				&Etag::from(""),
				&[&Etag::from("ANOTHER_ETAG")],
				true
			)
			.unwrap(),
			A.clone()
		);
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("A/AA"),
				&Etag::from(""),
				&[&Etag::from("ANOTHER_ETAG")],
				true
			)
			.unwrap(),
			AA.clone()
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			*get(&root_path, &ItemPath::from("A"), &Etag::from(""), &[], true)
				.unwrap_err()
				.downcast::<GetError>()
				.unwrap(),
			GetError::Conflict {
				item_path: ItemPath::from("A/")
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("A/AA/"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::Conflict {
				item_path: ItemPath::from("A/AA")
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("A/AC/not_exists"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::Conflict {
				item_path: ItemPath::from("A/AC")
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("A/not_exists"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NotFound {
				item_path: ItemPath::from("A/not_exists")
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("A/not_exists/nested"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NotFound {
				item_path: ItemPath::from("A/not_exists/")
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("B/not_exists"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NotFound {
				item_path: ItemPath::from("B/not_exists")
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("not_exists/"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NotFound {
				item_path: ItemPath::from("not_exists/")
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("not_exists"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NotFound {
				item_path: ItemPath::from("not_exists")
			}
		);
		/*
		// useless with `ItemPath`
		assert_eq!(
			*get(&root_path, &ItemPath::from("."), &Etag::from(""), &[], true)
				.unwrap_err()
				.downcast::<GetError>()
				.unwrap(),
			GetError::IncorrectItemName {
				item_path: ItemPath::from("."),
				error: String::from("`.` is not allowed")
			}
		);
		*/
		assert_eq!(
			get(&root_path, &ItemPath::from("."), &Etag::from(""), &[], true).unwrap(),
			root_without_public.clone()
		);
		/*
		// useless with `ItemPath`
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("A/.."),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IncorrectItemName {
				item_path: ItemPath::from("A/.."),
				error: String::from("`..` is not allowed")
			}
		);
		*/
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("A/.."),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap(),
			root_without_public.clone()
		);
		/*
		// useless with `ItemPath`
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("A/../"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IncorrectItemName {
				item_path: ItemPath::from("A/../"),
				error: String::from("`..` is not allowed")
			}
		);
		*/
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("A/../"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap(),
			root_without_public.clone()
		);
		/*
		// useless with `ItemPath` :
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("A/../AA"),
				&Etag::from(""),
				&[],
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
		*/
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("A/A\0A"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IncorrectItemName {
				item_path: ItemPath::from("A/A\0A"),
				error: format!("`{}` should not contains `\\0` character", "A\0A")
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("public/"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::CanNotBeListed {
				item_path: ItemPath::from("public/")
			},
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("public/C/"),
				&Etag::from(""),
				&[],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::CanNotBeListed {
				item_path: ItemPath::from("public/C/")
			}
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from(""),
				&Etag::from("ANOTHER_ETAG"),
				&[],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NoIfMatch {
				item_path: ItemPath::from(""),
				search: Etag::from("ANOTHER_ETAG"),
				found: root.get_etag().clone()
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("A/"),
				&Etag::from("ANOTHER_ETAG"),
				&[],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NoIfMatch {
				item_path: ItemPath::from("A/"),
				search: Etag::from("ANOTHER_ETAG"),
				found: A.get_etag().clone()
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("A/AA"),
				&Etag::from("ANOTHER_ETAG"),
				&[],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NoIfMatch {
				item_path: ItemPath::from("A/AA"),
				search: Etag::from("ANOTHER_ETAG"),
				found: AA.get_etag().clone()
			}
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from(""),
				&Etag::from(""),
				&[&Etag::from("*")],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IfNoneMatch {
				item_path: ItemPath::from(""),
				search: Etag::from("*"),
				found: root.get_etag().clone()
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("A/"),
				&Etag::from(""),
				&[&Etag::from("*")],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IfNoneMatch {
				item_path: ItemPath::from("A/"),
				search: Etag::from("*"),
				found: A.get_etag().clone()
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("A/AA"),
				&Etag::from(""),
				&[&Etag::from("*")],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IfNoneMatch {
				item_path: ItemPath::from("A/AA"),
				search: Etag::from("*"),
				found: AA.get_etag().clone()
			}
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from(""),
				&Etag::from(""),
				&[root.get_etag()],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IfNoneMatch {
				item_path: ItemPath::from(""),
				search: root.get_etag().clone(),
				found: root.get_etag().clone()
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("A/"),
				&Etag::from(""),
				&[A.get_etag()],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IfNoneMatch {
				item_path: ItemPath::from("A/"),
				search: A.get_etag().clone(),
				found: A.get_etag().clone()
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("A/AA"),
				&Etag::from(""),
				&[AA.get_etag()],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IfNoneMatch {
				item_path: ItemPath::from("A/AA"),
				search: AA.get_etag().clone(),
				found: AA.get_etag().clone()
			}
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			get(&root_path, &ItemPath::from(""), &Etag::from(""), &[], false).unwrap(),
			root_without_public.empty_clone()
		);
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("A/"),
				&Etag::from(""),
				&[],
				false
			)
			.unwrap(),
			A.empty_clone()
		);
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("A/AA"),
				&Etag::from(""),
				&[],
				false
			)
			.unwrap(),
			AA.empty_clone()
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("public/"),
				&Etag::from(""),
				&[],
				false
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::CanNotBeListed {
				item_path: ItemPath::from("public/")
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("public/C/"),
				&Etag::from(""),
				&[],
				false
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::CanNotBeListed {
				item_path: ItemPath::from("public/C/")
			}
		);
		assert_eq!(
			get(
				&root_path,
				&ItemPath::from("public/C/CA"),
				&Etag::from(""),
				&[],
				false
			)
			.unwrap(),
			CA.empty_clone()
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("public/not_exists"),
				&Etag::from(""),
				&[],
				false
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NotFound {
				item_path: ItemPath::from("public/not_exists")
			}
		);
		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("public/not_exists/"),
				&Etag::from(""),
				&[],
				false
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::CanNotBeListed {
				item_path: ItemPath::from("public/not_exists/")
			}
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			*get(
				&root_path,
				&ItemPath::from("A/.AA.itemdata.toml"),
				&Etag::from(""),
				&[&Etag::from("*")],
				true
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::IsSystemFile
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		tmp_folder.close().unwrap();
	}
}
