mod error;
pub use error::*;

#[cfg(test)]
pub mod tests;

pub fn get(
	root_item: &crate::item::Item,
	path: &crate::item::ItemPath,
	if_match: &crate::item::Etag,
	if_none_match: &[&crate::item::Etag],
) -> Result<crate::item::Item, Box<dyn std::error::Error>> {
	let paths = path.parts_iter();

	let mut pending = Some(root_item);
	let mut cumulated_path = crate::item::ItemPath::from("");

	if path != &crate::item::ItemPath::from("") {
		for path_part in paths {
			if let Err(error) = path_part.check_validity(false) {
				return Err(Box::new(GetError::IncorrectItemName {
					item_path: cumulated_path.joined(path_part).unwrap(),
					error,
				}));
			}

			match pending {
				Some(crate::item::Item::Folder {
					content: Some(folder_content),
					..
				}) => {
					pending = folder_content.get(path_part.name()).map(|boxed| &**boxed);

					cumulated_path = cumulated_path.joined(path_part).unwrap();
				}
				Some(crate::item::Item::Document { .. }) => {
					return Err(Box::new(GetError::Conflict {
						item_path: cumulated_path.document_clone(),
					}));
				}
				Some(crate::item::Item::Folder { content: None, .. }) => {
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
			crate::item::Item::Folder {
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
			crate::item::Item::Document {
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
	root_item: &'a mut crate::item::Item,
	path: &crate::item::ItemPath,
	if_match: &crate::item::Etag,
	if_none_match: &[&crate::item::Etag],
) -> Result<&'a mut crate::item::Item, Box<dyn std::error::Error>> {
	let paths = path.parts_iter();

	let mut pending = Some(root_item);
	let mut cumulated_path = crate::item::ItemPath::from("");

	if path != &crate::item::ItemPath::from("") {
		for path_part in paths {
			if let Err(error) = path_part.check_validity(false) {
				return Err(Box::new(GetError::IncorrectItemName {
					item_path: cumulated_path.joined(path_part).unwrap(),
					error,
				}));
			}

			match pending {
				Some(crate::item::Item::Folder {
					content: Some(folder_content),
					..
				}) => {
					pending = folder_content
						.get_mut(path_part.name())
						.map(|boxed| &mut **boxed);

					cumulated_path = cumulated_path.joined(path_part).unwrap();
				}
				Some(crate::item::Item::Document { .. }) => {
					return Err(Box::new(GetError::Conflict {
						item_path: cumulated_path.document_clone(),
					}));
				}
				Some(crate::item::Item::Folder { content: None, .. }) => {
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
			crate::item::Item::Folder {
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
			crate::item::Item::Document {
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
