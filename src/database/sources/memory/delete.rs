pub fn delete(
	root_item: &mut crate::Item,
	path: &crate::ItemPath,
	if_match: &crate::Etag,
) -> Result<crate::Etag, Box<dyn std::error::Error>> {
	if path.is_folder() {
		return Err(Box::new(DeleteError::DoesNotWorksForFolders));
	}

	let mut cumulated_path = crate::ItemPath::from("");
	for path_part in path.parts_iter() {
		cumulated_path = cumulated_path.joined(path_part).unwrap();
		if let Err(error) = crate::item_name_is_ok(path_part.name()) {
			return Err(Box::new(DeleteError::IncorrectItemName {
				item_path: cumulated_path,
				error,
			}));
		}
	}

	cumulated_path = crate::ItemPath::from("");
	for path_part in path.parts_iter() {
		cumulated_path = cumulated_path.joined(path_part).unwrap();
		if root_item.get_child(&cumulated_path).is_none() {
			return Err(Box::new(DeleteError::NotFound {
				item_path: cumulated_path,
			}));
		}
	}

	let parent_path = path.parent().unwrap();
	match root_item.get_child_mut(&parent_path) {
		Some(crate::Item::Folder {
			content: Some(parent_content),
			..
		}) => match parent_content.get_mut(path.file_name()) {
			Some(found_item) => match &**found_item {
				crate::Item::Document {
					etag: found_etag, ..
				} => {
					if !if_match.is_empty() && if_match.trim() != "*" && if_match != found_etag {
						return Err(Box::new(DeleteError::NoIfMatch {
							item_path: path.clone(),
							search: if_match.clone(),
							found: found_etag.clone(),
						}));
					}
					let old_etag = found_etag.clone();

					parent_content.remove(path.file_name());

					{
						for path_part in path
							.ancestors()
							.into_iter()
							.take(path.ancestors().len().saturating_sub(1))
							.rev()
						{
							if let Some(crate::Item::Folder {
								content: Some(parent_content),
								etag,
							}) = root_item.get_child_mut(&path_part)
							{
								let mut to_delete = vec![];
								for (child_name, child_item) in &*parent_content {
									if let crate::Item::Folder {
										content: Some(child_content),
										..
									} = &**child_item
									{
										if child_content.is_empty() {
											to_delete.push(child_name.clone());
										}
									}
								}

								for child_name in to_delete {
									parent_content.remove(&child_name);
								}

								*etag = crate::Etag::new();
							}
						}
					}

					return Ok(old_etag);
				}
				crate::Item::Folder { .. } => {
					if cumulated_path == *path {
						return Err(Box::new(DeleteError::Conflict {
							item_path: cumulated_path.folder_clone(),
						}));
					} else {
						return Err(Box::new(DeleteError::DoesNotWorksForFolders));
					}
				}
			},
			None => {
				return Err(Box::new(DeleteError::NotFound {
					item_path: path.clone(),
				}));
			}
		},
		Some(crate::Item::Folder { content: None, .. }) => {
			return Err(Box::new(DeleteError::NoContentInside {
				item_path: parent_path,
			}));
		}
		Some(crate::Item::Document { .. }) => {
			return Err(Box::new(DeleteError::Conflict {
				item_path: parent_path,
			}));
		}
		None => {
			return Err(Box::new(DeleteError::NotFound {
				item_path: parent_path,
			}));
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum DeleteError {
	Conflict {
		item_path: crate::ItemPath,
	},
	DoesNotWorksForFolders,
	NotFound {
		item_path: crate::ItemPath,
	},
	NoContentInside {
		item_path: crate::ItemPath,
	},
	IncorrectItemName {
		item_path: crate::ItemPath,
		error: String,
	},
	NoIfMatch {
		item_path: crate::ItemPath,
		search: crate::Etag,
		found: crate::Etag,
	},
}
impl std::fmt::Display for DeleteError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Conflict { item_path } => f.write_fmt(format_args!("name conflict between folder and file on the path `{}`", item_path)),
			Self::DoesNotWorksForFolders => f.write_str("this method does not works on folders"),
			Self::NotFound { item_path } => f.write_fmt(format_args!("path not found : `{}`", item_path)),
			Self::NoContentInside { item_path } => f.write_fmt(format_args!("no content found in `{}`", item_path)),
			Self::IncorrectItemName { item_path, error } => f.write_fmt(format_args!("the path `{}` is incorrect, because {}", item_path, error)),
			Self::NoIfMatch { item_path, search, found } => f.write_fmt(format_args!("the requested `{}` etag (through `IfMatch`) for `{}` was not found, found `{}` instead", search, item_path, found)),
		}
	}
}
impl std::error::Error for DeleteError {}
#[cfg(feature = "server_bin")]
impl crate::database::Error for DeleteError {
	fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse {
		match self {
			Self::Conflict { item_path: _ } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::CONFLICT,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::DoesNotWorksForFolders => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::NotFound { item_path: _ } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::NoContentInside { item_path: _ } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::IncorrectItemName {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::NoIfMatch {
				item_path: _,
				search: _,
				found: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(non_snake_case)]

	use super::{delete, DeleteError};
	use crate::{Etag, Item, ItemPath};

	// TODO : test last_modified

	fn build_test_db() -> (Item, Etag, Etag, Etag, Etag, Etag, Etag) {
		let root = Item::new_folder(vec![
			(
				"A",
				Item::new_folder(vec![
					(
						"AA",
						Item::new_folder(vec![(
							"AAA",
							Item::new_folder(vec![("AAAA", Item::new_doc(b"AAAA", "text/plain"))]),
						)]),
					),
					("AB", Item::new_doc(b"AB", "text/plain")),
				]),
			),
			(
				"public",
				Item::new_folder(vec![(
					"C",
					Item::new_folder(vec![(
						"CC",
						Item::new_folder(vec![("CCC", Item::new_doc(b"CCC", "text/plain"))]),
					)]),
				)]),
			),
		]);

		if let Item::Folder {
			etag: root_etag,
			content: Some(content),
		} = &root
		{
			if let Item::Folder {
				etag: A_etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				let AB_etag = if let Item::Document { etag, .. } = &**content.get("AB").unwrap() {
					etag
				} else {
					panic!();
				};

				if let Item::Folder {
					etag: AA_etag,
					content: Some(content),
				} = &**content.get("AA").unwrap()
				{
					if let Item::Folder {
						etag: AAA_etag,
						content: Some(content),
					} = &**content.get("AAA").unwrap()
					{
						if let Item::Document {
							etag: AAAA_etag, ..
						} = &**content.get("AAAA").unwrap()
						{
							return (
								root.clone(),
								root_etag.clone(),
								A_etag.clone(),
								AA_etag.clone(),
								AAA_etag.clone(),
								AAAA_etag.clone(),
								AB_etag.clone(),
							);
						} else {
							panic!();
						}
					} else {
						panic!();
					}
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	}

	#[test]
	fn simple_delete_on_not_existing() {
		let mut root = Item::new_folder(vec![]);
		let root_etag = root.get_etag().clone();

		assert_eq!(
			*delete(&mut root, &ItemPath::from("A/AA/AAA/AAAA"), &Etag::from(""),)
				.unwrap_err()
				.downcast::<DeleteError>()
				.unwrap(),
			DeleteError::NotFound {
				item_path: ItemPath::from("A/")
			}
		);

		if let Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(content.is_empty());
		} else {
			panic!();
		}
	}

	#[test]
	fn simple_delete_on_existing() {
		let (mut root, root_etag, A_etag, _, _, AAAA_etag, AB_etag) = build_test_db();

		let old_AAAA_etag =
			delete(&mut root, &ItemPath::from("A/AA/AAA/AAAA"), &Etag::from("")).unwrap();

		assert_eq!(AAAA_etag, old_AAAA_etag);

		if let Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_ne!(etag, &A_etag);
				assert!(!content.is_empty());

				assert_eq!(content.get("AA"), None);

				if let Item::Document { etag, .. } = &**content.get("AB").unwrap() {
					assert_eq!(etag, &AB_etag);
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	}

	#[test]
	fn does_not_works_for_folders() {
		let (mut root, root_etag, A_etag, AA_etag, AAA_etag, AAAA_etag, _) = build_test_db();

		assert_eq!(
			*delete(&mut root, &ItemPath::from("A/AA/"), &Etag::from(""),)
				.unwrap_err()
				.downcast::<DeleteError>()
				.unwrap(),
			DeleteError::DoesNotWorksForFolders,
		);

		if let Item::Folder {
			etag,
			content: Some(content),
		} = &root
		{
			assert_eq!(etag, &root_etag);
			if let Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				if let Item::Folder {
					etag,
					content: Some(content),
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					if let Item::Folder {
						etag,
						content: Some(content),
					} = &**content.get("AAA").unwrap()
					{
						assert_eq!(etag, &AAA_etag);
						if let Item::Document { etag, .. } = &**content.get("AAAA").unwrap() {
							assert_eq!(etag, &AAAA_etag);
						} else {
							panic!();
						}
					} else {
						panic!();
					}
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	}

	#[test]
	fn delete_with_if_match_not_found() {
		let (mut root, root_etag, A_etag, AA_etag, AAA_etag, AAAA_etag, _) = build_test_db();

		assert_eq!(
			*delete(
				&mut root,
				&ItemPath::from("A/AA/AAA/AAAA"),
				&Etag::from("OTHER_ETAG"),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::NoIfMatch {
				item_path: ItemPath::from("A/AA/AAA/AAAA"),
				found: AAAA_etag.clone(),
				search: Etag::from("OTHER_ETAG")
			}
		);

		if let Item::Folder {
			etag,
			content: Some(content),
		} = &root
		{
			assert_eq!(etag, &root_etag);
			if let Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				if let Item::Folder {
					etag,
					content: Some(content),
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					if let Item::Folder {
						etag,
						content: Some(content),
					} = &**content.get("AAA").unwrap()
					{
						assert_eq!(etag, &AAA_etag);
						if let Item::Document { etag, .. } = &**content.get("AAAA").unwrap() {
							assert_eq!(etag, &AAAA_etag);
						} else {
							panic!();
						}
					} else {
						panic!();
					}
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	}

	#[test]
	fn delete_with_if_match_found() {
		let (mut root, root_etag, A_etag, _, _, AAAA_etag, _) = build_test_db();

		let old_AAAA_etag =
			delete(&mut root, &ItemPath::from("A/AA/AAA/AAAA"), &AAAA_etag).unwrap();

		assert_eq!(old_AAAA_etag, AAAA_etag);

		if let Item::Folder {
			etag,
			content: Some(content),
		} = &root
		{
			assert_ne!(etag, &root_etag);
			assert!(!content.is_empty());

			if let Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_ne!(etag, &A_etag);
				assert!(!content.is_empty());

				assert_eq!(content.get("AA"), None);
				assert!(content.get("AB").is_some());
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	}

	#[test]
	fn delete_with_if_match_all() {
		let (mut root, root_etag, A_etag, _, _, AAAA_etag, _) = build_test_db();

		let old_AAAA_etag = delete(
			&mut root,
			&ItemPath::from("A/AA/AAA/AAAA"),
			&Etag::from("*"),
		)
		.unwrap();

		assert_eq!(old_AAAA_etag, AAAA_etag);

		if let Item::Folder {
			etag,
			content: Some(content),
		} = &root
		{
			assert_ne!(etag, &root_etag);
			assert!(!content.is_empty());

			if let Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_ne!(etag, &A_etag);
				assert!(!content.is_empty());

				assert_eq!(content.get("AA"), None);
				assert!(content.get("AB").is_some());
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	}

	#[test]
	fn delete_with_existing_folder_conflict() {
		let (mut root, root_etag, A_etag, AA_etag, AAA_etag, _, _) = build_test_db();

		assert_eq!(
			*delete(&mut root, &ItemPath::from("A/AA"), &Etag::from(""),)
				.unwrap_err()
				.downcast::<DeleteError>()
				.unwrap(),
			DeleteError::Conflict {
				item_path: ItemPath::from("A/AA/")
			}
		);

		if let Item::Folder {
			etag,
			content: Some(content),
		} = &root
		{
			assert_eq!(etag, &root_etag);
			assert!(!content.is_empty());

			if let Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				assert!(!content.is_empty());

				if let Item::Folder {
					etag,
					content: Some(content),
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert!(!content.is_empty());

					if let Item::Folder {
						etag,
						content: Some(content),
					} = &**content.get("AAA").unwrap()
					{
						assert_eq!(etag, &AAA_etag);
						assert!(!content.is_empty());
					}
				}

				assert!(content.get("AB").is_some());
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	}

	#[test]
	fn delete_in_public() {
		let (mut root, root_etag, _, _, _, _, _) = build_test_db();

		delete(
			&mut root,
			&ItemPath::from("public/C/CC/CCC"),
			&Etag::from(""),
		)
		.unwrap();

		if let Item::Folder {
			etag,
			content: Some(content),
		} = &root
		{
			assert_ne!(etag, &root_etag);
			assert!(!content.is_empty());

			assert!(content.get("A").is_some());
			assert_eq!(content.get("public"), None);
		} else {
			panic!();
		}
	}

	#[test]
	fn delete_in_incorrect_path() {
		let mut root = Item::new_folder(vec![]);
		let root_etag = root.get_etag().clone();

		assert_eq!(
			*delete(&mut root, &ItemPath::from("A/A\0A"), &Etag::from(""),)
				.unwrap_err()
				.downcast::<DeleteError>()
				.unwrap(),
			DeleteError::IncorrectItemName {
				item_path: ItemPath::from("A/A\0A"),
				error: String::from("`A\0A` should not contains `\\0` character")
			}
		);

		if let Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(content.is_empty());
		} else {
			panic!();
		}
	}
}
