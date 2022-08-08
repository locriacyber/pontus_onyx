mod error;
pub use error::*;

#[cfg(test)]
pub mod tests;

pub fn get(
	root_folder_path: &std::path::Path,
	path: &crate::item::ItemPath,
	if_match: &crate::item::Etag,
	if_none_match: &[&crate::item::Etag],
	get_content: bool,
) -> Result<crate::item::Item, Box<dyn std::error::Error>> {
	if path.ends_with(".itemdata.toml") {
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
					match toml::from_slice::<crate::item::DataDocument>(&itemdata_file_content) {
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
										return Ok(crate::item::Item::Document {
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
								return Ok(crate::item::Item::Document {
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
					match toml::from_slice::<crate::item::DataFolder>(&itemdata_file_content) {
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
															let entry_item = get(
																root_folder_path,
																&path
																	.joined(&if dir_entry
																		.file_type()
																		.unwrap()
																		.is_dir()
																	{
																		crate::item::ItemPathPart::Folder(entry_name.clone())
																	} else {
																		crate::item::ItemPathPart::Document(entry_name.clone())
																	})
																	.unwrap(),
																&crate::item::Etag::from(""),
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

											return Ok(crate::item::Item::Folder {
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
								return Ok(crate::item::Item::Folder {
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
			&crate::item::Etag::from(""),
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
