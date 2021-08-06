mod error;
pub use error::*;

#[cfg(test)]
pub mod tests;

pub fn get(
	storage: &dyn super::Storage,
	prefix: &str,
	path: &crate::item::ItemPath,
	if_match: &crate::item::Etag,
	if_none_match: &[&crate::item::Etag],
	get_content: bool,
) -> Result<crate::item::Item, Box<dyn std::error::Error>> {
	if path.ends_with(".itemdata.json") {
		return Err(Box::new(GetError::IsSystemFile));
	}

	if path.starts_with("public/") && path.is_folder() {
		return Err(Box::new(GetError::CanNotBeListed {
			item_path: path.clone(),
		}));
	}

	if path != &crate::item::ItemPath::from("") {
		let mut cumulated_path = crate::item::ItemPath::from("");
		for part in path.parts_iter() {
			cumulated_path = cumulated_path.joined(part).unwrap();
			if let Err(error) = part.check_validity(true) {
				if path != &crate::item::ItemPath::from("") {
					return Err(Box::new(GetError::IncorrectItemName {
						item_path: cumulated_path,
						error,
					}));
				}
			}
		}
	}

	let folderdata_path =
		crate::item::ItemPath::from(format!("{}/{}.folder.itemdata.json", prefix, path,).as_str());

	match storage.get_item(&format!("{}", folderdata_path)) {
		Ok(Some(folderdata_content)) => {
			if !path.is_folder() {
				return Err(Box::new(GetError::Conflict {
					item_path: path.clone(),
				}));
			}

			let content = if get_content {
				let mut content = std::collections::HashMap::new();

				for i in 0..storage.length().unwrap() {
					let key = storage.key(i).unwrap().unwrap();

					if let Some(remain) = key.strip_prefix(&format!("{}/{}", prefix, path)) {
						if !remain.ends_with(".itemdata.json") {
							if !remain.contains('/') && !remain.contains('\\') {
								content.insert(
									String::from(remain),
									Box::new(
										get(
											storage,
											prefix,
											&crate::item::ItemPath::from(
												key.strip_prefix(&format!("{}/", prefix)).unwrap(),
											),
											&crate::item::Etag::from(""),
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
												&path.joined_folder(name).unwrap(),
												&crate::item::Etag::from(""),
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

			match serde_json::from_str::<crate::item::DataFolder>(&folderdata_content) {
				Ok(folderdata) => {
					if !if_match.is_empty() {
						let upper_if_match = if_match.trim().to_uppercase();
						if folderdata.etag.trim().to_uppercase() != upper_if_match
							&& upper_if_match != "*"
						{
							return Err(Box::new(GetError::NoIfMatch {
								item_path: path.clone(),
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
									item_path: path.clone(),
									search: (*search_etag).clone(),
									found: folderdata.etag,
								}));
							}
						}
					}

					return Ok(crate::item::Item::Folder {
						etag: folderdata.etag,
						content,
					});
				}
				Err(error) => {
					return Err(Box::new(GetError::CanNotSerializeFile {
						item_path: folderdata_path,
						error: format!("{}", error),
					}))
				}
			}
		}
		Ok(None) => {
			let target_parent = path
				.parent()
				.unwrap_or_else(|| crate::item::ItemPath::from(""));
			let filedata_path = crate::item::ItemPath::from(
				format!(
					"{}/{}{}",
					prefix,
					target_parent,
					format!(".{}.itemdata.json", path.file_name())
				)
				.as_str(),
			);

			match storage.get_item(&format!("{}", filedata_path)) {
				Ok(Some(filedata_content)) => {
					if path.is_folder() {
						return Err(Box::new(GetError::Conflict {
							item_path: path.document_clone(),
						}));
					}

					match serde_json::from_str::<crate::item::DataDocument>(&filedata_content) {
						Ok(filedata) => {
							let content = if get_content {
								match storage.get_item(&format!("{}/{}", prefix, path)) {
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
										item_path: path.clone(),
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
											item_path: path.clone(),
											search: (*search_etag).clone(),
											found: filedata.etag,
										}));
									}
								}
							}

							return Ok(crate::item::Item::Document {
								etag: filedata.etag,
								content_type: filedata.content_type,
								last_modified: filedata.last_modified,
								content,
							});
						}
						Err(error) => {
							return Err(Box::new(GetError::CanNotSerializeFile {
								item_path: filedata_path,
								error: format!("{}", error),
							}))
						}
					}
				}
				Ok(None) => {
					let filedata_path =
						format!("{}/{}.folder.itemdata.json", prefix, path.folder_clone(),);

					match storage.get_item(&filedata_path) {
						Ok(Some(_)) => {
							return Err(Box::new(GetError::Conflict {
								item_path: path.folder_clone(),
							}));
						}
						Ok(None) => {
							if path != &crate::item::ItemPath::from("") {
								let parent = path.parent().unwrap();

								let parent_get = get(
									storage,
									prefix,
									&parent,
									&crate::item::Etag::from(""),
									&[],
									false,
								);

								if let Ok(crate::item::Item::Document { .. }) = parent_get {
									return Err(Box::new(GetError::Conflict {
										item_path: parent.clone(),
									}));
								}

								if let Err(error) = parent_get {
									if let GetError::CanNotBeListed { item_path: _ } =
										error.downcast_ref::<GetError>().unwrap()
									{
										// nothing to do
									} else {
										return Err(error);
									}
								}
							}

							return Err(Box::new(GetError::NotFound {
								item_path: path.clone(),
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
