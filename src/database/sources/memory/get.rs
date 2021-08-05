pub fn get(
	root_item: &crate::Item,
	path: &crate::ItemPath,
	if_match: &crate::Etag,
	if_none_match: &[&crate::Etag],
) -> Result<crate::Item, Box<dyn std::error::Error>> {
	let paths = path.parts_iter();

	let mut pending = Some(root_item);
	let mut cumulated_path = crate::ItemPath::from("");

	if path != &crate::ItemPath::from("") {
		for path_part in paths {
			if let Err(error) = crate::item_name_is_ok(path_part.name()) {
				return Err(Box::new(GetError::IncorrectItemName {
					item_path: cumulated_path.joined(path_part).unwrap(),
					error,
				}));
			}

			match pending {
				Some(crate::Item::Folder {
					content: Some(folder_content),
					..
				}) => {
					pending = folder_content.get(path_part.name()).map(|boxed| &**boxed);

					cumulated_path = cumulated_path.joined(path_part).unwrap();
				}
				Some(crate::Item::Document { .. }) => {
					return Err(Box::new(GetError::Conflict {
						item_path: cumulated_path.document_clone(),
					}));
				}
				Some(crate::Item::Folder { content: None, .. }) => {
					return Err(Box::new(GetError::NoContentInside {
						item_path: cumulated_path.folder_clone(),
					}));
				}
				None => {
					return Err(Box::new(GetError::NotFound {
						item_path: cumulated_path,
					}));
				}
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
						return Err(Box::new(GetError::NoIfMatch {
							item_path: cumulated_path,
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
							return Err(Box::new(GetError::IfNoneMatch {
								item_path: cumulated_path,
								search: (*search_etag).clone(),
								found: found_etag.clone(),
							}));
						}
					}
				}

				if path.is_folder() {
					if path.starts_with("public/") {
						return Err(Box::new(GetError::CanNotBeListed {
							item_path: path.clone(),
						}));
					} else {
						Ok(item.clone()) // TODO : expensive clone here
					}
				} else {
					Err(Box::new(GetError::Conflict {
						item_path: cumulated_path.folder_clone(),
					}))
				}
			}
			crate::Item::Document {
				etag: found_etag, ..
			} => {
				if !if_match.is_empty() {
					let upper_if_match = if_match.trim().to_uppercase();
					if found_etag.trim().to_uppercase() != upper_if_match && upper_if_match != "*" {
						return Err(Box::new(GetError::NoIfMatch {
							item_path: cumulated_path,
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
							return Err(Box::new(GetError::IfNoneMatch {
								item_path: cumulated_path,
								search: (*search_etag).clone(),
								found: found_etag.clone(),
							}));
						}
					}
				}

				if !path.is_folder() {
					Ok(item.clone()) // TODO : expensive clone here
				} else {
					Err(Box::new(GetError::Conflict {
						item_path: cumulated_path.document_clone(),
					}))
				}
			}
		},
		None => Err(Box::new(GetError::NotFound {
			item_path: cumulated_path,
		})),
	}
}

pub fn get_internal_mut<'a>(
	root_item: &'a mut crate::Item,
	path: &crate::ItemPath,
	if_match: &crate::Etag,
	if_none_match: &[&crate::Etag],
) -> Result<&'a mut crate::Item, Box<dyn std::error::Error>> {
	let paths = path.parts_iter();

	let mut pending = Some(root_item);
	let mut cumulated_path = crate::ItemPath::from("");

	if path != &crate::ItemPath::from("") {
		for path_part in paths {
			if let Err(error) = crate::item_name_is_ok(path_part.name()) {
				return Err(Box::new(GetError::IncorrectItemName {
					item_path: cumulated_path.joined(path_part).unwrap(),
					error,
				}));
			}

			match pending {
				Some(crate::Item::Folder {
					content: Some(folder_content),
					..
				}) => {
					pending = folder_content
						.get_mut(path_part.name())
						.map(|boxed| &mut **boxed);

					cumulated_path = cumulated_path.joined(path_part).unwrap();
				}
				Some(crate::Item::Document { .. }) => {
					return Err(Box::new(GetError::Conflict {
						item_path: cumulated_path.document_clone(),
					}));
				}
				Some(crate::Item::Folder { content: None, .. }) => {
					return Err(Box::new(GetError::NoContentInside {
						item_path: cumulated_path.folder_clone(),
					}));
				}
				None => {
					return Err(Box::new(GetError::NotFound {
						item_path: cumulated_path,
					}));
				}
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
						return Err(Box::new(GetError::NoIfMatch {
							item_path: cumulated_path,
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
							return Err(Box::new(GetError::IfNoneMatch {
								item_path: cumulated_path,
								search: (*search_etag).clone(),
								found: found_etag.clone(),
							}));
						}
					}
				}

				if path.is_folder() {
					Ok(item)
				} else {
					Err(Box::new(GetError::Conflict {
						item_path: cumulated_path.folder_clone(),
					}))
				}
			}
			crate::Item::Document {
				etag: found_etag, ..
			} => {
				if !if_match.is_empty() {
					let upper_if_match = if_match.trim().to_uppercase();
					if found_etag.trim().to_uppercase() != upper_if_match && upper_if_match != "*" {
						return Err(Box::new(GetError::NoIfMatch {
							item_path: cumulated_path,
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
							return Err(Box::new(GetError::IfNoneMatch {
								item_path: cumulated_path,
								search: (*search_etag).clone(),
								found: found_etag.clone(),
							}));
						}
					}
				}

				if !path.is_folder() {
					Ok(item)
				} else {
					Err(Box::new(GetError::Conflict {
						item_path: cumulated_path.document_clone(),
					}))
				}
			}
		},
		None => Err(Box::new(GetError::NotFound {
			item_path: cumulated_path,
		})),
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
	NoContentInside {
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
}
impl std::fmt::Display for GetError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Conflict { item_path } => f.write_fmt(format_args!("name conflict between folder and file on the path `{}`", item_path)),
			Self::NotFound { item_path } => f.write_fmt(format_args!("path not found : `{}`", item_path)),
			Self::NoContentInside { item_path } => f.write_fmt(format_args!("no content found in `{}`", item_path)),
			Self::IncorrectItemName { item_path, error } => f.write_fmt(format_args!("the path `{}` is incorrect, because {}", item_path, error)),
			Self::CanNotBeListed { item_path } => f.write_fmt(format_args!("the folder `{:?}` can not be listed", item_path)),
			Self::NoIfMatch { item_path, search, found } => f.write_fmt(format_args!("the requested `{}` etag (through `IfMatch`) for `{}` was not found, found `{}` instead", search, item_path, found)),
			Self::IfNoneMatch { item_path, search, found } => f.write_fmt(format_args!("the unwanted etag `{}` (through `IfNoneMatch`) for `{}` was matches with `{}`", search, item_path, found)),
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
			Self::NoContentInside { item_path } => {
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
						actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
						None,
						Some(format!("{}", self)),
						should_have_body,
					)
				}
			}
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
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(non_snake_case)]

	use super::{get, GetError};
	use crate::{Etag, Item, ItemPath};

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

		assert_eq!(
			get(&root, &ItemPath::from(""), &Etag::from(""), &vec![]).unwrap(),
			root.clone() // TODO : should return root_without_public, but recursion in get make it buggy.
		);
		assert_eq!(
			get(&root, &ItemPath::from("A/"), &Etag::from(""), &vec![]).unwrap(),
			A.clone()
		);
		assert_eq!(
			get(&root, &ItemPath::from("A/AA"), &Etag::from(""), &vec![]).unwrap(),
			AA.clone()
		);
		assert_eq!(
			get(&root, &ItemPath::from("A/AB"), &Etag::from(""), &vec![]).unwrap(),
			AB
		);
		assert_eq!(
			get(&root, &ItemPath::from("A/AC"), &Etag::from(""), &vec![]).unwrap(),
			AC
		);
		assert_eq!(
			get(&root, &ItemPath::from("B/"), &Etag::from(""), &vec![]).unwrap(),
			B
		);
		assert_eq!(
			get(&root, &ItemPath::from("B/BA"), &Etag::from(""), &vec![]).unwrap(),
			BA
		);
		assert_eq!(
			get(&root, &ItemPath::from("B/BB"), &Etag::from(""), &vec![]).unwrap(),
			BB
		);
		assert_eq!(
			get(
				&root,
				&ItemPath::from("public/C/CA"),
				&Etag::from(""),
				&vec![]
			)
			.unwrap(),
			CA
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			get(&root, &ItemPath::from(""), root.get_etag(), &vec![]).unwrap(),
			root.clone() // TODO : should return root_without_public, but recursion in get make it buggy.
		);
		assert_eq!(
			get(&root, &ItemPath::from("A/"), A.get_etag(), &vec![]).unwrap(),
			A.clone()
		);
		assert_eq!(
			get(&root, &ItemPath::from("A/AA"), AA.get_etag(), &vec![]).unwrap(),
			AA.clone()
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			get(
				&root,
				&ItemPath::from(""),
				&Etag::from(""),
				&[&Etag::from("ANOTHER_ETAG")]
			)
			.unwrap(),
			root.clone() // TODO : should return root_without_public, but recursion in get make it buggy.
		);
		assert_eq!(
			get(
				&root,
				&ItemPath::from("A/"),
				&Etag::from(""),
				&[&Etag::from("ANOTHER_ETAG")]
			)
			.unwrap(),
			A.clone()
		);
		assert_eq!(
			get(
				&root,
				&ItemPath::from("A/AA"),
				&Etag::from(""),
				&[&Etag::from("ANOTHER_ETAG")]
			)
			.unwrap(),
			AA.clone()
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			*get(&root, &ItemPath::from("A"), &Etag::from(""), &vec![])
				.unwrap_err()
				.downcast::<GetError>()
				.unwrap(),
			GetError::Conflict {
				item_path: ItemPath::from("A/")
			}
		);
		assert_eq!(
			*get(&root, &ItemPath::from("A/AA/"), &Etag::from(""), &vec![])
				.unwrap_err()
				.downcast::<GetError>()
				.unwrap(),
			GetError::Conflict {
				item_path: ItemPath::from("A/AA")
			}
		);
		assert_eq!(
			*get(
				&root,
				&ItemPath::from("A/AC/not_exists"),
				&Etag::from(""),
				&vec![]
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
				&root,
				&ItemPath::from("A/not_exists"),
				&Etag::from(""),
				&vec![]
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
				&root,
				&ItemPath::from("A/not_exists/nested"),
				&Etag::from(""),
				&vec![]
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
				&root,
				&ItemPath::from("B/not_exists"),
				&Etag::from(""),
				&vec![]
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
				&root,
				&ItemPath::from("not_exists/"),
				&Etag::from(""),
				&vec![]
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
				&root,
				&ItemPath::from("not_exists"),
				&Etag::from(""),
				&vec![]
			)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
			GetError::NotFound {
				item_path: ItemPath::from("not_exists")
			}
		);
		/*
		useless with `ItemPath`
		assert_eq!(
			*get(&root, &ItemPath::from("."), &Etag::from(""), &vec![])
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
			get(&root, &ItemPath::from("."), &Etag::from(""), &vec![]).unwrap(),
			root.clone()
		);
		/*
		useless with `ItemPath`
		assert_eq!(
			*get(&root, &ItemPath::from("A/.."), &Etag::from(""), &vec![])
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
			get(&root, &ItemPath::from("A/.."), &Etag::from(""), &vec![]).unwrap(),
			root.clone(),
		);
		/*
		useless with `ItemPath`
		assert_eq!(
			*get(&root, &ItemPath::from("A/../"), &Etag::from(""), &vec![])
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
			get(&root, &ItemPath::from("A/../"), &Etag::from(""), &vec![]).unwrap(),
			root.clone(),
		);
		/*
		useless with `ItemPath`
		assert_eq!(
			*get(&root, &ItemPath::from("A/../AA"), &Etag::from(""), &vec![])
				.unwrap_err()
				.downcast::<GetError>()
				.unwrap(),
			GetError::IncorrectItemName {
				item_path: ItemPath::from("A/../"),
				error: String::from("`..` is not allowed")
			}
		);
		*/
		/*
		// useless with `ItemPath` :
		assert_eq!(
			*get(&root, &ItemPath::from("A/../AA"), &Etag::from(""), &vec![])
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
			*get(&root, &ItemPath::from("A/A\0A"), &Etag::from(""), &vec![])
				.unwrap_err()
				.downcast::<GetError>()
				.unwrap(),
			GetError::IncorrectItemName {
				item_path: ItemPath::from("A/A\0A"),
				error: format!("`{}` should not contains `\\0` character", "A\0A")
			}
		);
		assert_eq!(
			*get(&root, &ItemPath::from("public/"), &Etag::from(""), &vec![])
				.unwrap_err()
				.downcast::<GetError>()
				.unwrap(),
			GetError::CanNotBeListed {
				item_path: ItemPath::from("public/")
			},
		);
		assert_eq!(
			*get(
				&root,
				&ItemPath::from("public/C/"),
				&Etag::from(""),
				&vec![]
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
				&root,
				&ItemPath::from(""),
				&Etag::from("ANOTHER_ETAG"),
				&vec![]
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
				&root,
				&ItemPath::from("A/"),
				&Etag::from("ANOTHER_ETAG"),
				&vec![]
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
				&root,
				&ItemPath::from("A/AA"),
				&Etag::from("ANOTHER_ETAG"),
				&vec![]
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
				&root,
				&ItemPath::from(""),
				&Etag::from(""),
				&[&Etag::from("*")]
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
				&root,
				&ItemPath::from("A/"),
				&Etag::from(""),
				&[&Etag::from("*")]
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
				&root,
				&ItemPath::from("A/AA"),
				&Etag::from(""),
				&[&Etag::from("*")]
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
				&root,
				&ItemPath::from(""),
				&Etag::from(""),
				&[root.get_etag()]
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
				&root,
				&ItemPath::from("A/"),
				&Etag::from(""),
				&[A.get_etag()]
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
				&root,
				&ItemPath::from("A/AA"),
				&Etag::from(""),
				&[AA.get_etag()]
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
	}
}
