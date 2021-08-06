mod error;
pub use error::*;

#[cfg(test)]
pub mod tests;

pub fn put(
	root_item: &mut crate::item::Item,
	path: &crate::item::ItemPath,
	if_match: &crate::item::Etag,
	if_none_match: &[&crate::item::Etag],
	item: crate::item::Item,
) -> crate::database::PutResult {
	if path.is_folder() {
		return crate::database::PutResult::Err(Box::new(PutError::DoesNotWorksForFolders));
	}

	let mut cumultated_path = crate::item::ItemPath::from("");
	for path_part in path.parts_iter() {
		cumultated_path = cumultated_path.joined(path_part).unwrap();
		if let Err(error) = path_part.check_validity(false) {
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
				if let Some(crate::item::Item::Folder {
					content: Some(content),
					..
				}) = root_item.get_child_mut(&path_part.parent().unwrap())
				{
					content.insert(
						String::from(path_part.file_name()),
						Box::new(crate::item::Item::new_folder(vec![])),
					);
				}
			}
		}
	}

	match super::get::get_internal_mut(root_item, path, if_match, if_none_match) {
		Ok(found) => {
			if let crate::item::Item::Document {
				etag,
				content,
				content_type,
				last_modified,
			} = found
			{
				if let crate::item::Item::Document {
					content: new_content,
					content_type: new_content_type,
					..
				} = item
				{
					let new_etag = crate::item::Etag::new();

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
								Some(crate::item::Item::Folder { etag, .. }) => {
									*etag = crate::item::Etag::new();
								}
								Some(crate::item::Item::Document { etag, .. }) => {
									*etag = crate::item::Etag::new();
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
					&crate::item::Etag::from(""),
					&[],
				) {
					Ok(parent_folder) => match parent_folder {
						crate::item::Item::Folder {
							content: Some(content),
							..
						} => {
							if let crate::item::Item::Document {
								content: new_content,
								content_type: new_content_type,
								..
							} = item
							{
								let new_etag = crate::item::Etag::new();
								let new_item = crate::item::Item::Document {
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
											Some(crate::item::Item::Folder { etag, .. }) => {
												*etag = crate::item::Etag::new();
											}
											Some(crate::item::Item::Document { etag, .. }) => {
												*etag = crate::item::Etag::new();
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
						crate::item::Item::Folder { content: None, .. } => {
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
