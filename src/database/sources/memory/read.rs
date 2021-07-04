pub fn read(
	root_item: &crate::Item,
	path: &std::path::Path,
	if_match: &crate::Etag,
	if_none_match: &[&crate::Etag],
) -> Result<crate::Item, Box<dyn std::any::Any>> {
	let path = path
		.to_str()
		.unwrap()
		.strip_prefix('/')
		.unwrap_or_else(|| path.to_str().unwrap());
	let paths: Vec<String> = path.split('/').map(String::from).collect();

	let requested_is_folder = match paths.last() {
		Some(part) => part.is_empty(),
		None => true,
	};
	let paths_len = paths.len();
	let paths = if requested_is_folder {
		paths
			.into_iter()
			.take(paths_len.saturating_sub(1))
			.collect()
	} else {
		paths
	};

	if requested_is_folder && path.starts_with("public/") {
		return Err(Box::new(ReadError::CanNotBeListed));
	}

	let mut pending = Some(root_item);
	let mut cumulated_path = String::from("");

	for (path_id, path_part) in paths.iter().enumerate() {
		if let Err(error) = crate::database::utils::is_ok(&path_part) {
			return Err(Box::new(ReadError::IncorrectItemName {
				item_path: format!(
					"{}{}{}",
					cumulated_path,
					path_part,
					if requested_is_folder || path_id < (paths.len() - 1) {
						"/"
					} else {
						""
					}
				)
				.into(),
				error,
			}));
		}

		match pending {
			Some(crate::Item::Folder {
				content: Some(folder_content),
				..
			}) => {
				pending = folder_content.get(path_part).map(|boxed| &**boxed);

				cumulated_path += &format!(
					"{}{}",
					path_part,
					if (path_id + 1) == paths.len() && !requested_is_folder {
						""
					} else {
						"/"
					}
				);
			}
			Some(crate::Item::Document { .. }) => {
				return Err(Box::new(ReadError::Conflict {
					item_path: std::path::PathBuf::from(cumulated_path),
				}));
			}
			Some(crate::Item::Folder { content: None, .. }) => {
				return Err(Box::new(ReadError::NoContentInside {
					item_path: std::path::PathBuf::from(cumulated_path),
				}));
			}
			None => {
				return Err(Box::new(ReadError::NotFound {
					item_path: std::path::PathBuf::from(cumulated_path),
				}));
			}
		}
	}

	match pending {
		Some(item) => match item {
			crate::Item::Folder {
				etag: found_etag, ..
			} => {
				if !if_match.is_empty() {
					let upper_if_match = if_match.trim().to_uppercase();
					if found_etag.trim().to_uppercase() != upper_if_match && upper_if_match != "*" {
						return Err(Box::new(ReadError::NoIfMatch {
							item_path: std::path::PathBuf::from(cumulated_path),
							search: if_match.clone(),
							found: found_etag.clone(),
						}));
					}
				}

				if !if_none_match.is_empty() {
					for search_etag in if_none_match {
						if found_etag.trim().to_uppercase() == search_etag.trim().to_uppercase()
							|| search_etag.trim() == "*"
						{
							return Err(Box::new(ReadError::IfNoneMatch {
								item_path: std::path::PathBuf::from(cumulated_path),
								search: (*search_etag).clone(),
								found: found_etag.clone(),
							}));
						}
					}
				}

				if requested_is_folder {
					Ok(item.clone()) // TODO : expensive clone here
				} else {
					Err(Box::new(ReadError::Conflict {
						item_path: std::path::PathBuf::from(cumulated_path),
					}))
				}
			}
			crate::Item::Document {
				etag: found_etag, ..
			} => {
				if !if_match.is_empty() {
					let upper_if_match = if_match.trim().to_uppercase();
					if found_etag.trim().to_uppercase() != upper_if_match && upper_if_match != "*" {
						return Err(Box::new(ReadError::NoIfMatch {
							item_path: std::path::PathBuf::from(cumulated_path),
							search: if_match.clone(),
							found: found_etag.clone(),
						}));
					}
				}

				if !if_none_match.is_empty() {
					for search_etag in if_none_match {
						if found_etag.trim().to_uppercase() == search_etag.trim().to_uppercase()
							|| search_etag.trim() == "*"
						{
							return Err(Box::new(ReadError::IfNoneMatch {
								item_path: std::path::PathBuf::from(cumulated_path),
								search: (*search_etag).clone(),
								found: found_etag.clone(),
							}));
						}
					}
				}

				if !requested_is_folder {
					Ok(item.clone()) // TODO : expensive clone here
				} else {
					Err(Box::new(ReadError::Conflict {
						item_path: std::path::PathBuf::from(cumulated_path),
					}))
				}
			}
		},
		None => Err(Box::new(ReadError::NotFound {
			item_path: std::path::PathBuf::from(cumulated_path),
		})),
	}
}

pub fn read_internal_mut<'a>(
	root_item: &'a mut crate::Item,
	path: &std::path::Path,
	if_match: &crate::Etag,
	if_none_match: &[&crate::Etag],
) -> Result<&'a mut crate::Item, ReadError> {
	let path = path.strip_prefix("/").unwrap_or(&path).to_str().unwrap();
	let paths: Vec<String> = path.split('/').map(String::from).collect();

	let requested_is_folder = match paths.last() {
		Some(part) => part.is_empty(),
		None => true,
	};
	let paths_len = paths.len();
	let paths = if requested_is_folder {
		paths
			.into_iter()
			.take(paths_len.saturating_sub(1))
			.collect()
	} else {
		paths
	};

	let mut pending = Some(root_item);
	let mut cumulated_path = String::from("");

	for (path_id, path_part) in paths.iter().enumerate() {
		if let Err(error) = crate::database::utils::is_ok(path_part) {
			return Err(ReadError::IncorrectItemName {
				item_path: format!(
					"{}{}{}",
					cumulated_path,
					path_part,
					if requested_is_folder || path_id < (paths.len() - 1) {
						"/"
					} else {
						""
					}
				)
				.into(),
				error,
			});
		}

		match pending {
			Some(crate::Item::Folder {
				content: Some(folder_content),
				..
			}) => {
				pending = folder_content.get_mut(path_part).map(|boxed| &mut **boxed);

				cumulated_path += &format!(
					"{}{}",
					path_part,
					if (path_id + 1) == paths.len() && !requested_is_folder {
						""
					} else {
						"/"
					}
				);
			}
			Some(crate::Item::Document { .. }) => {
				return Err(ReadError::Conflict {
					item_path: std::path::PathBuf::from(cumulated_path),
				});
			}
			Some(crate::Item::Folder { content: None, .. }) => {
				return Err(ReadError::NoContentInside {
					item_path: std::path::PathBuf::from(cumulated_path),
				});
			}
			None => {
				return Err(ReadError::NotFound {
					item_path: std::path::PathBuf::from(cumulated_path),
				});
			}
		}
	}

	match pending {
		Some(item) => match item {
			crate::Item::Folder {
				etag: found_etag, ..
			} => {
				if !if_match.is_empty() {
					let upper_if_match = if_match.trim().to_uppercase();
					if found_etag.trim().to_uppercase() != upper_if_match && upper_if_match != "*" {
						return Err(ReadError::NoIfMatch {
							item_path: std::path::PathBuf::from(cumulated_path),
							search: if_match.clone(),
							found: found_etag.clone(),
						});
					}
				}

				if !if_none_match.is_empty() {
					for search_etag in if_none_match {
						if found_etag.trim().to_uppercase() == search_etag.trim().to_uppercase()
							|| search_etag.trim() == "*"
						{
							return Err(ReadError::IfNoneMatch {
								item_path: std::path::PathBuf::from(cumulated_path),
								search: (*search_etag).clone(),
								found: found_etag.clone(),
							});
						}
					}
				}

				if requested_is_folder {
					Ok(item)
				} else {
					Err(ReadError::Conflict {
						item_path: std::path::PathBuf::from(cumulated_path),
					})
				}
			}
			crate::Item::Document {
				etag: found_etag, ..
			} => {
				if !if_match.is_empty() {
					let upper_if_match = if_match.trim().to_uppercase();
					if found_etag.trim().to_uppercase() != upper_if_match && upper_if_match != "*" {
						return Err(ReadError::NoIfMatch {
							item_path: std::path::PathBuf::from(cumulated_path),
							search: if_match.clone(),
							found: found_etag.clone(),
						});
					}
				}

				if !if_none_match.is_empty() {
					for search_etag in if_none_match {
						if found_etag.trim().to_uppercase() == search_etag.trim().to_uppercase()
							|| search_etag.trim() == "*"
						{
							return Err(ReadError::IfNoneMatch {
								item_path: std::path::PathBuf::from(cumulated_path),
								search: (*search_etag).clone(),
								found: found_etag.clone(),
							});
						}
					}
				}

				if !requested_is_folder {
					Ok(item)
				} else {
					Err(ReadError::Conflict {
						item_path: std::path::PathBuf::from(cumulated_path),
					})
				}
			}
		},
		None => Err(ReadError::NotFound {
			item_path: std::path::PathBuf::from(cumulated_path),
		}),
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReadError {
	Conflict {
		item_path: std::path::PathBuf,
	},
	NotFound {
		item_path: std::path::PathBuf,
	},
	NoContentInside {
		item_path: std::path::PathBuf,
	},
	IncorrectItemName {
		item_path: std::path::PathBuf,
		error: String,
	},
	CanNotBeListed,
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
}
impl std::fmt::Display for ReadError {
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
impl std::error::Error for ReadError {}
impl crate::database::Error for ReadError {
	fn to_response(&self, _: &str, _: bool) -> actix_web::HttpResponse {
		todo!() // TODO
	}
}

#[cfg(test)]
mod tests {
	#![allow(non_snake_case)]

	use super::{read, ReadError};

	#[test]
	fn bq0ydkqyvmkzfd3d78() {
		let AA = crate::Item::new_doc(b"AA", "text/plain");
		let AB = crate::Item::new_doc(b"AB", "text/plain");
		let AC = crate::Item::new_doc(b"AC", "text/plain");
		let BA = crate::Item::new_doc(b"BA", "text/plain");
		let BB = crate::Item::new_doc(b"BB", "text/plain");
		let CC = crate::Item::new_doc(b"CC", "text/plain");

		let A = crate::Item::new_folder(vec![
			("AA", AA.clone()),
			("AB", AB.clone()),
			("AC", AC.clone()),
		]);
		let B = crate::Item::new_folder(vec![("BA", BA.clone()), ("BB", BB.clone())]);
		let C = crate::Item::new_folder(vec![("CC", CC.clone())]);
		let public = crate::Item::new_folder(vec![("C", C.clone())]);

		let root = crate::Item::new_folder(vec![
			("A", A.clone()),
			("B", B.clone()),
			("public", public.clone()),
		]);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			read(
				&root,
				&std::path::PathBuf::from(""),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap(),
			root.clone()
		);
		assert_eq!(
			read(
				&root,
				&std::path::PathBuf::from("A/"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap(),
			A.clone()
		);
		assert_eq!(
			read(
				&root,
				&std::path::PathBuf::from("A/AA"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap(),
			AA.clone()
		);
		assert_eq!(
			read(
				&root,
				&std::path::PathBuf::from("A/AB"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap(),
			AB
		);
		assert_eq!(
			read(
				&root,
				&std::path::PathBuf::from("A/AC"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap(),
			AC
		);
		assert_eq!(
			read(
				&root,
				&std::path::PathBuf::from("B/"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap(),
			B
		);
		assert_eq!(
			read(
				&root,
				&std::path::PathBuf::from("B/BA"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap(),
			BA
		);
		assert_eq!(
			read(
				&root,
				&std::path::PathBuf::from("B/BB"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap(),
			BB
		);
		assert_eq!(
			read(
				&root,
				&std::path::PathBuf::from("public/C/CC"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap(),
			CC
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			read(
				&root,
				&std::path::PathBuf::from(""),
				root.get_etag(),
				&vec![]
			)
			.unwrap(),
			root.clone()
		);
		assert_eq!(
			read(
				&root,
				&std::path::PathBuf::from("A/"),
				A.get_etag(),
				&vec![]
			)
			.unwrap(),
			A.clone()
		);
		assert_eq!(
			read(
				&root,
				&std::path::PathBuf::from("A/AA"),
				AA.get_etag(),
				&vec![]
			)
			.unwrap(),
			AA.clone()
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			read(
				&root,
				&std::path::PathBuf::from(""),
				&crate::Etag::from(""),
				&[&crate::Etag::from("ANOTHER_ETAG")]
			)
			.unwrap(),
			root.clone()
		);
		assert_eq!(
			read(
				&root,
				&std::path::PathBuf::from("A/"),
				&crate::Etag::from(""),
				&[&crate::Etag::from("ANOTHER_ETAG")]
			)
			.unwrap(),
			A.clone()
		);
		assert_eq!(
			read(
				&root,
				&std::path::PathBuf::from("A/AA"),
				&crate::Etag::from(""),
				&[&crate::Etag::from("ANOTHER_ETAG")]
			)
			.unwrap(),
			AA.clone()
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("A"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::Conflict {
				item_path: std::path::PathBuf::from("A")
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("A/AA/"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::Conflict {
				item_path: std::path::PathBuf::from("A/AA/")
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("A/AC/not_exists"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::Conflict {
				item_path: std::path::PathBuf::from("A/AC/")
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("A/not_exists"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::NotFound {
				item_path: std::path::PathBuf::from("A/not_exists")
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("A/not_exists/nested"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::NotFound {
				item_path: std::path::PathBuf::from("A/not_exists/")
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("B/not_exists"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::NotFound {
				item_path: std::path::PathBuf::from("B/not_exists")
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("not_exists/"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::NotFound {
				item_path: std::path::PathBuf::from("not_exists/")
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("not_exists"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::NotFound {
				item_path: std::path::PathBuf::from("not_exists")
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("."),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::IncorrectItemName {
				item_path: std::path::PathBuf::from("."),
				error: String::from("`.` is not allowed"),
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("A/.."),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/.."),
				error: String::from("`..` is not allowed"),
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("A/../"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/../"),
				error: String::from("`..` is not allowed"),
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("A/../AA"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/../"),
				error: String::from("`..` is not allowed"),
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("A/A\0A"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/A\0A"),
				error: format!("`{}` should not contains \\0 character", "A\0A"),
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("public/"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::CanNotBeListed
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("public/C/"),
				&crate::Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::CanNotBeListed
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from(""),
				&crate::Etag::from("ANOTHER_ETAG"),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::NoIfMatch {
				item_path: std::path::PathBuf::from(""),
				search: crate::Etag::from("ANOTHER_ETAG"),
				found: root.get_etag().clone()
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("A/"),
				&crate::Etag::from("ANOTHER_ETAG"),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::NoIfMatch {
				item_path: std::path::PathBuf::from("A/"),
				search: crate::Etag::from("ANOTHER_ETAG"),
				found: A.get_etag().clone()
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("A/AA"),
				&crate::Etag::from("ANOTHER_ETAG"),
				&vec![]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::NoIfMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				search: crate::Etag::from("ANOTHER_ETAG"),
				found: AA.get_etag().clone()
			}
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from(""),
				&crate::Etag::from(""),
				&[&crate::Etag::from("*")]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::IfNoneMatch {
				item_path: std::path::PathBuf::from(""),
				search: crate::Etag::from("*"),
				found: root.get_etag().clone()
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("A/"),
				&crate::Etag::from(""),
				&[&crate::Etag::from("*")]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::IfNoneMatch {
				item_path: std::path::PathBuf::from("A/"),
				search: crate::Etag::from("*"),
				found: A.get_etag().clone()
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("A/AA"),
				&crate::Etag::from(""),
				&[&crate::Etag::from("*")]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::IfNoneMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				search: crate::Etag::from("*"),
				found: AA.get_etag().clone()
			}
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from(""),
				&crate::Etag::from(""),
				&[root.get_etag()]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::IfNoneMatch {
				item_path: std::path::PathBuf::from(""),
				search: root.get_etag().clone(),
				found: root.get_etag().clone()
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("A/"),
				&crate::Etag::from(""),
				&[A.get_etag()]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::IfNoneMatch {
				item_path: std::path::PathBuf::from("A/"),
				search: A.get_etag().clone(),
				found: A.get_etag().clone()
			}
		);
		assert_eq!(
			*read(
				&root,
				&std::path::PathBuf::from("A/AA"),
				&crate::Etag::from(""),
				&[AA.get_etag()]
			)
			.unwrap_err()
			.downcast::<ReadError>()
			.unwrap(),
			ReadError::IfNoneMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				search: AA.get_etag().clone(),
				found: AA.get_etag().clone()
			}
		);
	}
}
