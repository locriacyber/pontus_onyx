pub fn put(
	root_item: &mut crate::Item,
	path: &crate::ItemPath,
	if_match: &crate::Etag,
	if_none_match: &[&crate::Etag],
	item: crate::Item,
) -> crate::database::PutResult {
	if path.is_folder() {
		return crate::database::PutResult::Err(Box::new(PutError::DoesNotWorksForFolders));
	}

	let mut cumultated_path = crate::ItemPath::from("");
	for path_part in path.parts_iter() {
		cumultated_path = cumultated_path.joined(path_part).unwrap();
		if let Err(error) = crate::item_name_is_ok(path_part.name()) {
			return crate::database::PutResult::Err(Box::new(PutError::GetError(
				super::GetError::IncorrectItemName {
					item_path: cumultated_path,
					error,
				},
			)));
		}
	}

	{
		for path_part in path
			.ancestors()
			.into_iter()
			.take(path.ancestors().len().saturating_sub(1))
		{
			if root_item.get_child(&path_part).is_none() {
				if let Some(crate::Item::Folder {
					content: Some(content),
					..
				}) = root_item.get_child_mut(&path_part.parent().unwrap())
				{
					content.insert(
						String::from(path_part.file_name()),
						Box::new(crate::Item::new_folder(vec![])),
					);
				}
			}
		}
	}

	match super::get::get_internal_mut(root_item, path, if_match, if_none_match) {
		Ok(found) => {
			if let crate::Item::Document {
				etag,
				content,
				content_type,
				last_modified,
			} = found
			{
				if let crate::Item::Document {
					content: new_content,
					content_type: new_content_type,
					..
				} = item
				{
					let new_etag = crate::Etag::new();

					if if_match.trim() != "" && (etag != if_match && if_match != "*") {
						return crate::database::PutResult::Err(Box::new(PutError::GetError(
							super::GetError::NoIfMatch {
								item_path: path.clone(),
								found: etag.clone(),
								search: if_match.clone(),
							},
						)));
					}

					if content_type == &new_content_type && content == &new_content {
						return crate::database::PutResult::Err(Box::new(
							PutError::ContentNotChanged,
						));
					}

					*etag = new_etag.clone();
					*last_modified = chrono::Utc::now();
					*content_type = new_content_type;
					*content = new_content;

					{
						let ancestors_len = path.ancestors().len();

						for path_part in path
							.ancestors()
							.into_iter()
							.take(ancestors_len.saturating_sub(1))
						{
							match root_item.get_child_mut(&path_part) {
								Some(crate::Item::Folder { etag, .. }) => {
									*etag = crate::Etag::new();
								}
								Some(crate::Item::Document { etag, .. }) => {
									*etag = crate::Etag::new();
								}
								None => {
									return crate::database::PutResult::Err(Box::new(
										PutError::InternalError,
									));
								}
							}
						}
					}

					return crate::database::PutResult::Updated(new_etag);
				} else {
					return crate::database::PutResult::Err(Box::new(
						PutError::DoesNotWorksForFolders,
					));
				}
			} else {
				return crate::database::PutResult::Err(Box::new(PutError::DoesNotWorksForFolders));
			}
		}
		Err(error) => match *error.downcast::<super::GetError>().unwrap() {
			super::GetError::NotFound { .. } => {
				match super::get::get_internal_mut(
					root_item,
					&path.parent().unwrap(),
					&crate::Etag::from(""),
					&[],
				) {
					Ok(parent_folder) => match parent_folder {
						crate::Item::Folder {
							content: Some(content),
							..
						} => {
							if let crate::Item::Document {
								content: new_content,
								content_type: new_content_type,
								..
							} = item
							{
								let new_etag = crate::Etag::new();
								let new_item = crate::Item::Document {
									etag: new_etag.clone(),
									content: new_content,
									content_type: new_content_type,
									last_modified: chrono::Utc::now(),
								};
								content.insert(String::from(path.file_name()), Box::new(new_item));

								{
									let ancestors_len = path.ancestors().len();

									for path_part in path
										.ancestors()
										.into_iter()
										.take(ancestors_len.saturating_sub(1))
									{
										match root_item.get_child_mut(&path_part) {
											Some(crate::Item::Folder { etag, .. }) => {
												*etag = crate::Etag::new();
											}
											Some(crate::Item::Document { etag, .. }) => {
												*etag = crate::Etag::new();
											}
											None => {
												return crate::database::PutResult::Err(Box::new(
													PutError::InternalError,
												));
											}
										}
									}
								}

								return crate::database::PutResult::Created(new_etag);
							} else {
								return crate::database::PutResult::Err(Box::new(
									PutError::DoesNotWorksForFolders,
								));
							}
						}
						crate::Item::Folder { content: None, .. } => {
							return crate::database::PutResult::Err(Box::new(
								PutError::NoContentInside {
									item_path: path.clone(),
								},
							));
						}
						_ => {
							return crate::database::PutResult::Err(Box::new(
								PutError::InternalError,
							));
						}
					},
					Err(error) => {
						let error = *error.downcast::<super::GetError>().unwrap();
						return crate::database::PutResult::Err(Box::new(PutError::GetError(
							error,
						)));
					}
				}
			}
			super::GetError::CanNotBeListed { .. } => {
				return crate::database::PutResult::Err(Box::new(PutError::DoesNotWorksForFolders));
			}
			error => {
				return crate::database::PutResult::Err(Box::new(PutError::GetError(error)));
			}
		},
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum PutError {
	GetError(super::GetError),
	NoContentInside {
		item_path: crate::ItemPath,
	},
	DoesNotWorksForFolders,
	InternalError,
	ContentNotChanged,
	CanNotFetchParent {
		item_path: crate::ItemPath,
		error: super::GetError,
	},
}
impl std::fmt::Display for PutError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::GetError(error) => f.write_fmt(format_args!("{}", error)),
			Self::NoContentInside { item_path } => {
				f.write_fmt(format_args!("no content found in `{}`", item_path))
			}
			Self::DoesNotWorksForFolders => f.write_str("this method does not works on folders"),
			Self::InternalError => f.write_str("internal server error"),
			Self::ContentNotChanged => f.write_str("content not changed"),
			Self::CanNotFetchParent { item_path, error } => f.write_fmt(format_args!(
				"can not fetch parent of `{}`, because : `{}`",
				item_path, error
			)),
		}
	}
}
impl std::error::Error for PutError {}
#[cfg(feature = "server_bin")]
impl crate::database::Error for PutError {
	fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse {
		match self {
			Self::GetError(error) => error.to_response(origin, should_have_body),
			Self::NoContentInside { item_path: _ } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::DoesNotWorksForFolders => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::InternalError => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
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
			Self::CanNotFetchParent {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
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

	use super::{put, PutError};
	use crate::{Etag, Item, ItemPath};

	// TODO : test last_modified

	fn build_test_db() -> (Item, crate::Etag, crate::Etag, crate::Etag) {
		let root = Item::new_folder(vec![(
			"A",
			Item::new_folder(vec![("AA", Item::new_doc(b"AA", "text/plain"))]),
		)]);

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
				if let Item::Document { etag: AA_etag, .. } = &**content.get("AA").unwrap() {
					return (
						root.clone(),
						root_etag.clone(),
						A_etag.clone(),
						AA_etag.clone(),
					);
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
	}

	#[test]
	fn simple_put_on_not_existing() {
		let mut root = Item::new_folder(vec![]);
		let root_etag = root.get_etag().clone();

		let AA_etag = put(
			&mut root,
			&ItemPath::from("AA"),
			&Etag::from(""),
			&[],
			Item::new_doc(b"AA", "text/plain"),
		)
		.unwrap();

		if let Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let Item::Document {
				etag,
				content: Some(content),
				content_type,
				..
			} = &**content.get("AA").unwrap()
			{
				assert_eq!(etag, &AA_etag);
				assert_eq!(content, &b"AA".to_vec());
				assert_eq!(content_type, &crate::ContentType::from("text/plain"));
			}
		} else {
			panic!();
		};
	}

	#[test]
	fn simple_put_on_existing() {
		let (mut root, root_etag, A_etag, old_AA_etag) = build_test_db();

		let AA_etag = put(
			&mut root,
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&[],
			Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap();

		assert_ne!(old_AA_etag, AA_etag);

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

				if let Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA2".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain2"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
	}

	#[test]
	fn content_not_changed() {
		let (mut root, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			*put(
				&mut root,
				&ItemPath::from("A/AA"),
				&Etag::from(""),
				&[],
				Item::new_doc(b"AA", "text/plain")
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::ContentNotChanged
		);

		if let Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(!content.is_empty());

			if let Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				assert!(!content.is_empty());

				if let Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
	}

	#[test]
	fn does_not_works_for_folders() {
		let mut root = Item::new_folder(vec![]);
		let root_etag = root.get_etag().clone();

		assert_eq!(
			*put(
				&mut root,
				&ItemPath::from(""),
				&Etag::from(""),
				&[],
				Item::new_folder(vec![])
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::DoesNotWorksForFolders
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
		};
	}

	#[test]
	fn put_with_if_none_match_all_on_not_existing() {
		let mut root = Item::new_folder(vec![]);
		let root_etag = root.get_etag().clone();

		let AA_etag = put(
			&mut root,
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&[&Etag::from("*")],
			Item::new_doc(b"AA", "text/plain"),
		)
		.unwrap();

		if let Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let Item::Folder {
				content: Some(content),
				..
			} = &**content.get("A").unwrap()
			{
				assert!(!content.is_empty());

				if let Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
	}

	#[test]
	fn put_with_if_none_match_all_on_existing() {
		let (mut root, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			*put(
				&mut root,
				&ItemPath::from("A/AA"),
				&Etag::from(""),
				&[&Etag::from("*")],
				Item::new_doc(b"AA2", "text/plain2"),
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(super::super::GetError::IfNoneMatch {
				item_path: ItemPath::from("A/AA"),
				found: AA_etag.clone(),
				search: Etag::from("*")
			})
		);

		if let Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(!content.is_empty());

			if let Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				assert!(!content.is_empty());

				if let Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
	}

	#[test]
	fn put_with_if_match_not_found() {
		let (mut root, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			*put(
				&mut root,
				&ItemPath::from("A/AA"),
				&Etag::from("ANOTHER_ETAG"),
				&[],
				Item::new_doc(b"AA2", "text/plain2"),
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(super::super::GetError::NoIfMatch {
				item_path: ItemPath::from("A/AA"),
				found: AA_etag.clone(),
				search: Etag::from("ANOTHER_ETAG")
			})
		);

		if let Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(!content.is_empty());

			if let Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				assert!(!content.is_empty());

				if let Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
	}

	#[test]
	fn put_with_if_match_found() {
		let (mut root, root_etag, A_etag, mut AA_etag) = build_test_db();

		AA_etag = put(
			&mut root,
			&ItemPath::from("A/AA"),
			&AA_etag,
			&[],
			Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap();

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

				if let Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA2".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain2"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
	}

	#[test]
	fn put_with_if_match_all() {
		let (mut root, root_etag, A_etag, old_AA_etag) = build_test_db();

		let AA_etag = put(
			&mut root,
			&ItemPath::from("A/AA"),
			&Etag::from("*"),
			&[],
			Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap();

		assert_ne!(old_AA_etag, AA_etag);

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

				if let Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA2".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain2"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
	}

	#[test]
	fn put_with_existing_document_conflict() {
		let (mut root, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			*put(
				&mut root,
				&ItemPath::from("A/AA/AAA"),
				&Etag::from(""),
				&[],
				Item::new_doc(b"AAA", "text/plain")
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(super::super::GetError::Conflict {
				item_path: ItemPath::from("A/AA")
			})
		);

		if let Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(!content.is_empty());

			if let Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				assert!(!content.is_empty());

				if let Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
	}

	#[test]
	fn put_with_existing_folder_conflict() {
		let (mut root, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			*put(
				&mut root,
				&ItemPath::from("A"),
				&Etag::from(""),
				&[],
				Item::new_doc(b"A", "text/plain")
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(super::super::GetError::Conflict {
				item_path: ItemPath::from("A/")
			})
		);

		if let Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(!content.is_empty());

			if let Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				assert!(!content.is_empty());

				if let Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
	}

	#[test]
	fn put_in_public() {
		let mut root = Item::new_folder(vec![]);
		let root_etag = root.get_etag().clone();

		let AA_etag = put(
			&mut root,
			&ItemPath::from("public/A/AA"),
			&Etag::from(""),
			&[],
			Item::new_doc(b"AA", "text/plain"),
		)
		.unwrap();

		if let Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let Item::Folder {
				content: Some(content),
				..
			} = &**content.get("public").unwrap()
			{
				assert!(!content.is_empty());
				if let Item::Folder {
					content: Some(content),
					..
				} = &**content.get("A").unwrap()
				{
					assert!(!content.is_empty());

					if let Item::Document {
						etag,
						content: Some(content),
						content_type,
						..
					} = &**content.get("AA").unwrap()
					{
						assert_eq!(etag, &AA_etag);
						assert_eq!(content, &b"AA".to_vec());
						assert_eq!(content_type, &crate::ContentType::from("text/plain"));
					}
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
	}

	#[test]
	fn put_in_incorrect_path() {
		let mut root = Item::new_folder(vec![]);
		let root_etag = root.get_etag().clone();

		assert_eq!(
			*put(
				&mut root,
				&ItemPath::from("A/A\0A"),
				&Etag::from(""),
				&[],
				Item::new_doc(b"AA2", "text/plain2"),
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(super::super::GetError::IncorrectItemName {
				item_path: ItemPath::from("A/A\0A"),
				error: String::from("`A\0A` should not contains `\\0` character")
			})
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
		};
	}
}
