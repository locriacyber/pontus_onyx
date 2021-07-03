pub fn put(
	root_item: &mut crate::Item,
	path: &std::path::Path,
	if_match: &crate::Etag,
	if_none_match: &[&crate::Etag],
	item: crate::Item,
) -> Result<crate::Etag, PutError> {
	let mut cumultated_path = std::path::PathBuf::new();
	for path_part in path {
		cumultated_path = cumultated_path.join(path_part);
		if let Err(error) = crate::database::utils::is_ok(path_part.to_str().unwrap()) {
			return Err(PutError::IncorrectItemName {
				item_path: cumultated_path,
				error,
			});
		}
	}

	{
		let parents = {
			let ancestors = path.ancestors();
			let mut paths: Vec<&std::path::Path> = ancestors.into_iter().collect();
			paths = paths
				.into_iter()
				.rev()
				.skip(1)
				.take(
					ancestors
						.count()
						.checked_sub(1)
						.unwrap_or(0)
						.checked_sub(1)
						.unwrap_or(0),
				)
				.collect();
			paths
		};

		for path_part in parents {
			if let None = root_item.get_child(path_part) {
				if let Some(parent) = root_item.get_child_mut(path_part.parent().unwrap()) {
					if let crate::Item::Folder {
						content: Some(content),
						..
					} = parent
					{
						content.insert(
							path_part.file_name().unwrap().to_str().unwrap().to_string(),
							Box::new(crate::Item::new_folder(vec![])),
						);
					}
				}
			}
		}
	}

	match super::read_internal_mut(root_item, path, if_match, if_none_match) {
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

					if if_match.trim() != "" {
						if etag != if_match && if_match != "*" {
							return Err(PutError::NoIfMatch {
								item_path: path.strip_prefix("/").unwrap_or(&path).into(),
								found: etag.clone(),
								search: if_match.clone(),
							});
						}
					}

					if content_type == &new_content_type && content == &new_content {
						return Err(PutError::ContentNotChanged);
					}

					*etag = new_etag.clone();
					*last_modified = chrono::Utc::now();
					*content_type = new_content_type;
					*content = new_content;

					{
						let parents = {
							let ancestors = path.ancestors();
							let mut paths: Vec<&std::path::Path> = ancestors.into_iter().collect();
							paths = paths
								.into_iter()
								.rev()
								.take(ancestors.count().checked_sub(1).unwrap_or(0))
								.collect();
							paths
						};

						for path_part in parents {
							match root_item.get_child_mut(path_part) {
								Some(crate::Item::Folder { etag, .. }) => {
									*etag = crate::Etag::new();
								}
								Some(crate::Item::Document { etag, .. }) => {
									*etag = crate::Etag::new();
								}
								None => {
									return Err(PutError::InternalError);
								}
							}
						}
					}

					return Ok(new_etag);
				} else {
					return Err(PutError::DoesNotWorksForFolders);
				}
			} else {
				return Err(PutError::DoesNotWorksForFolders);
			}
		}
		Err(super::ReadError::NotFound { .. }) => {
			match super::read_internal_mut(
				root_item,
				&crate::database::utils::get_parent(path),
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
							content.insert(
								String::from(path.file_name().unwrap().to_str().unwrap()),
								Box::new(new_item),
							);

							{
								let parents = {
									let ancestors = path.ancestors();
									let mut paths: Vec<&std::path::Path> =
										ancestors.into_iter().collect();
									paths = paths
										.into_iter()
										.rev()
										.take(ancestors.count().checked_sub(1).unwrap_or(0))
										.collect();
									paths
								};

								for path_part in parents {
									match root_item.get_child_mut(path_part) {
										Some(crate::Item::Folder { etag, .. }) => {
											*etag = crate::Etag::new();
										}
										Some(crate::Item::Document { etag, .. }) => {
											*etag = crate::Etag::new();
										}
										None => {
											return Err(PutError::InternalError);
										}
									}
								}
							}

							return Ok(new_etag);
						}
					}
					crate::Item::Folder { content: None, .. } => {
						return Err(PutError::MissingContent {
							item_path: path.to_path_buf(),
						});
					}
					_ => {
						return Err(PutError::InternalError);
					}
				},
				Err(error) => {
					return Err(PutError::CanNotFetchParent {
						item_path: path.to_path_buf(),
						error,
					});
				}
			}
		}
		Err(super::ReadError::CanNotBeListed) => {
			return Err(PutError::DoesNotWorksForFolders);
		}
		Err(super::ReadError::IncorrectItemName { item_path, error }) => {
			return Err(PutError::IncorrectItemName { item_path, error });
		}
		Err(super::ReadError::Conflict { item_path }) => {
			return Err(PutError::Conflict { item_path });
		}
		Err(super::ReadError::NoContentInside { item_path }) => {
			return Err(PutError::NoContentInside { item_path });
		}
		Err(super::ReadError::NoIfMatch {
			found,
			item_path,
			search,
		}) => {
			return Err(PutError::NoIfMatch {
				found,
				item_path,
				search,
			});
		}
		Err(super::ReadError::MissingContent { item_path }) => {
			return Err(PutError::MissingContent { item_path });
		}
		Err(super::ReadError::IfNoneMatch {
			item_path,
			search,
			found,
		}) => {
			return Err(PutError::IfNoneMatch {
				item_path,
				search,
				found,
			});
		}
	}

	unreachable!()
}

#[derive(Debug, PartialEq, Eq)]
pub enum PutError {
	Conflict {
		item_path: std::path::PathBuf,
	},
	NoContentInside {
		item_path: std::path::PathBuf,
	},
	IncorrectItemName {
		item_path: std::path::PathBuf,
		error: String,
	},
	DoesNotWorksForFolders,
	InternalError,
	ContentNotChanged,
	IfNoneMatch {
		item_path: std::path::PathBuf,
		search: crate::Etag,
		found: crate::Etag,
	},
	NoIfMatch {
		item_path: std::path::PathBuf,
		search: crate::Etag,
		found: crate::Etag,
	},
	CanNotFetchParent {
		item_path: std::path::PathBuf,
		error: super::ReadError,
	},
	MissingContent {
		item_path: std::path::PathBuf,
	},
}
impl std::fmt::Display for PutError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		/*
		match self {
			Self::Conflict{item_path} => f.write_fmt(format_args!("name conflict between folder and file on the path `{}`", item_path)),
			Self::NotFound{item_path} => f.write_fmt(format_args!("path not found : `{}`", item_path)),
			Self::NoContentInside{item_path} => f.write_fmt(format_args!("no content found in `{}`", item_path)),
		}
		*/
		f.write_str("TODO")
	}
}
// TODO : public_display (without details)
// TODO : to_http_response
impl std::error::Error for PutError {}

#[cfg(test)]
mod tests {
	#![allow(non_snake_case)]

	use super::{put, PutError};

	// TODO : test last_modified

	fn build_test_db() -> (crate::Item, crate::Etag, crate::Etag, crate::Etag) {
		let root = crate::Item::new_folder(vec![(
			"A",
			crate::Item::new_folder(vec![("AA", crate::Item::new_doc(b"AA", "text/plain"))]),
		)]);

		if let crate::Item::Folder {
			etag: root_etag,
			content: Some(content),
		} = &root
		{
			if let crate::Item::Folder {
				etag: A_etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				if let crate::Item::Document { etag: AA_etag, .. } = &**content.get("AA").unwrap() {
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
		let mut root = crate::Item::new_folder(vec![]);
		let root_etag = root.get_etag().clone();

		let AA_etag = put(
			&mut root,
			&std::path::PathBuf::from("AA"),
			&crate::Etag::from(""),
			&[],
			crate::Item::new_doc(b"AA", "text/plain"),
		)
		.unwrap();

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Document {
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
			&std::path::PathBuf::from("A/AA"),
			&crate::Etag::from(""),
			&[],
			crate::Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap();

		assert_ne!(old_AA_etag, AA_etag);

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_ne!(etag, &A_etag);
				assert!(!content.is_empty());

				if let crate::Item::Document {
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
			put(
				&mut root,
				&std::path::PathBuf::from("A/AA"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"AA", "text/plain")
			),
			Err(PutError::ContentNotChanged)
		);

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				assert!(!content.is_empty());

				if let crate::Item::Document {
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
		let mut root = crate::Item::new_folder(vec![]);
		let root_etag = root.get_etag().clone();

		assert_eq!(
			put(
				&mut root,
				&std::path::PathBuf::from(""),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_folder(vec![])
			),
			Err(PutError::DoesNotWorksForFolders)
		);
		if let crate::Item::Folder {
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
		let mut root = crate::Item::new_folder(vec![]);
		let root_etag = root.get_etag().clone();

		let AA_etag = put(
			&mut root,
			&std::path::PathBuf::from("A/AA"),
			&crate::Etag::from(""),
			&[&crate::Etag::from("*")],
			crate::Item::new_doc(b"AA", "text/plain"),
		)
		.unwrap();

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				content: Some(content),
				..
			} = &**content.get("A").unwrap()
			{
				assert!(!content.is_empty());

				if let crate::Item::Document {
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
			put(
				&mut root,
				&std::path::PathBuf::from("A/AA"),
				&crate::Etag::from(""),
				&[&crate::Etag::from("*")],
				crate::Item::new_doc(b"AA", "text/plain"),
			),
			Err(PutError::IfNoneMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				found: AA_etag.clone(),
				search: crate::Etag::from("*"),
			})
		);

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				assert!(!content.is_empty());

				if let crate::Item::Document {
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
			put(
				&mut root,
				&std::path::PathBuf::from("A/AA"),
				&crate::Etag::from("ANOTHER_ETAG"),
				&[],
				crate::Item::new_doc(b"AA2", "text/plain2"),
			),
			Err(PutError::NoIfMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				found: AA_etag.clone(),
				search: crate::Etag::from("ANOTHER_ETAG"),
			})
		);

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				assert!(!content.is_empty());

				if let crate::Item::Document {
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
			&std::path::PathBuf::from("A/AA"),
			&AA_etag,
			&[],
			crate::Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap();

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_ne!(etag, &A_etag);
				assert!(!content.is_empty());

				if let crate::Item::Document {
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
		let (mut root, root_etag, A_etag, mut AA_etag) = build_test_db();

		AA_etag = put(
			&mut root,
			&std::path::PathBuf::from("A/AA"),
			&crate::Etag::from("*"),
			&[],
			crate::Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap();

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_ne!(etag, &A_etag);
				assert!(!content.is_empty());

				if let crate::Item::Document {
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
	fn put_with_existing_folder_conflict() {
		let (mut root, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			put(
				&mut root,
				&std::path::PathBuf::from("A"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"A", "text/plain")
			),
			Err(PutError::Conflict {
				item_path: std::path::PathBuf::from("A")
			})
		);

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				assert!(!content.is_empty());

				if let crate::Item::Document {
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
		let mut root = crate::Item::new_folder(vec![]);
		let root_etag = root.get_etag().clone();

		let AA_etag = put(
			&mut root,
			&std::path::PathBuf::from("public/A/AA"),
			&crate::Etag::from(""),
			&[],
			crate::Item::new_doc(b"AA", "text/plain"),
		)
		.unwrap();

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				content: Some(content),
				..
			} = &**content.get("public").unwrap()
			{
				assert!(!content.is_empty());
				if let crate::Item::Folder {
					content: Some(content),
					..
				} = &**content.get("A").unwrap()
				{
					assert!(!content.is_empty());

					if let crate::Item::Document {
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
		let mut root = crate::Item::new_folder(vec![]);
		let root_etag = root.get_etag().clone();

		assert_eq!(
			put(
				&mut root,
				&std::path::PathBuf::from("A/../AA"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"AA", "text/plain"),
			),
			Err(PutError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/../"),
				error: String::from("`..` is not allowed")
			})
		);

		if let crate::Item::Folder {
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
