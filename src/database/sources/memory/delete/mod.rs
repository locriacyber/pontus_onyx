mod error;
pub use error::*;

#[cfg(test)]
pub mod tests;

pub fn delete(
	root_item: &mut crate::item::Item,
	path: &crate::item::ItemPath,
	if_match: &crate::item::Etag,
) -> Result<crate::item::Etag, Box<dyn std::error::Error>> {
	if path.is_folder() {
		return Err(Box::new(DeleteError::DoesNotWorksForFolders));
	}

	let mut cumulated_path = crate::item::ItemPath::from("");
	for path_part in path.parts_iter() {
		cumulated_path = cumulated_path.joined(path_part).unwrap();
		if let Err(error) = path_part.check_validity(false) {
			return Err(Box::new(DeleteError::IncorrectItemName {
				item_path: cumulated_path,
				error,
			}));
		}
	}

	cumulated_path = crate::item::ItemPath::from("");
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
		Some(crate::item::Item::Folder {
			content: Some(parent_content),
			..
		}) => match parent_content.get_mut(path.file_name()) {
			Some(found_item) => match &**found_item {
				crate::item::Item::Document {
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
							if let Some(crate::item::Item::Folder {
								content: Some(parent_content),
								etag,
							}) = root_item.get_child_mut(&path_part)
							{
								let mut to_delete = vec![];
								for (child_name, child_item) in &*parent_content {
									if let crate::item::Item::Folder {
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

								*etag = crate::item::Etag::new();
							}
						}
					}

					return Ok(old_etag);
				}
				crate::item::Item::Folder { .. } => {
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
		Some(crate::item::Item::Folder { content: None, .. }) => {
			return Err(Box::new(DeleteError::NoContentInside {
				item_path: parent_path,
			}));
		}
		Some(crate::item::Item::Document { .. }) => {
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
