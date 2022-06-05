mod error;
pub use error::*;

#[cfg(test)]
pub mod tests;

pub fn put(
	storage: &dyn super::Storage,
	prefix: &str,
	path: &crate::item::ItemPath,
	if_match: &crate::item::Etag,
	if_none_match: &[&crate::item::Etag],
	item: crate::item::Item,
) -> crate::database::PutResult {
	match super::get(storage, prefix, path, if_match, if_none_match, true) {
		Ok(crate::item::Item::Document {
			content_type: old_content_type,
			content: old_content,
			..
		}) => {
			if let crate::item::Item::Document {
				content_type: new_content_type,
				content: new_content,
				last_modified: new_last_modified,
				..
			} = item
			{
				if new_content != old_content || new_content_type != old_content_type {
					match new_content {
						Some(new_content) => {
							let new_etag = crate::item::Etag::new();

							let serialized_data =
								serde_json::to_string(&crate::item::DataDocument {
									datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
									etag: new_etag.clone(),
									last_modified: Some(time::OffsetDateTime::now_utc()),
									content_type: new_content_type,
								});

							let filedata_path = crate::item::ItemPath::from(
								format!(
									"{}/{}/.{}.itemdata.json",
									prefix,
									path.parent().unwrap(),
									path.file_name()
								)
								.as_str(),
							);
							match serialized_data {
								Ok(serialized_data) => {
									if storage
										.set_item(&format!("{}", filedata_path), &serialized_data)
										.is_err()
									{
										return crate::database::PutResult::Err(Box::new(
											PutError::GetError(super::GetError::CanNotGetStorage),
										));
									}
									if storage
										.set_item(
											&format!("{}/{}", prefix, path),
											&base64::encode(new_content),
										)
										.is_err()
									{
										return crate::database::PutResult::Err(Box::new(
											PutError::GetError(super::GetError::CanNotGetStorage),
										));
									}

									for ancestor in path
										.ancestors()
										.into_iter()
										.take(path.ancestors().len().saturating_sub(1))
									{
										let folderdata_path =
											crate::item::ItemPath::from(
												format!(
													"{}/{}.folder.itemdata.json",
													prefix, ancestor,
												)
												.as_str(),
											);

										match storage.get_item(&format!("{}", folderdata_path)) {
											Ok(Some(folderdata_content)) => {
												match serde_json::from_str::<crate::item::DataFolder>(
													&folderdata_content,
												) {
													Ok(folderdata) => {
														let mut new_folderdata = folderdata.clone();
														new_folderdata.etag =
															crate::item::Etag::new();

														match serde_json::to_string(&new_folderdata) {
														Ok(new_folderdata_content) => {
															if storage.set_item(&format!("{}", folderdata_path), &new_folderdata_content).is_err() {
																return crate::database::PutResult::Err(Box::new(PutError::GetError(super::GetError::CanNotGetStorage)));
															}
														}
														Err(error) => return crate::database::PutResult::Err(Box::new(PutError::CanNotSerializeFile { item_path: folderdata_path, error: format!("{}", error) })),
													}
													}
													Err(error) => {
														return crate::database::PutResult::Err(
															Box::new(
																PutError::CanNotDeserializeFile {
																	item_path: folderdata_path,
																	error: format!("{}", error),
																},
															),
														)
													}
												}
											}
											Ok(None) => {
												// it should not happen
												// we can not fix it easily here because risks or conflict
												return crate::database::PutResult::Err(Box::new(
													PutError::InternalError,
												));
											}
											Err(_) => {
												return crate::database::PutResult::Err(Box::new(
													PutError::GetError(
														super::GetError::CanNotGetStorage,
													),
												));
											}
										}
									}

									return crate::database::PutResult::Updated(
										new_etag,
										new_last_modified
											.unwrap_or_else(time::OffsetDateTime::now_utc),
									);
								}
								Err(error) => {
									return crate::database::PutResult::Err(Box::new(
										PutError::CanNotDeserializeFile {
											item_path: filedata_path,
											error: format!("{}", error),
										},
									));
								}
							}
						}
						None => {
							return crate::database::PutResult::Err(Box::new(
								PutError::NoContentInside {
									item_path: path.clone(),
								},
							));
						}
					}
				} else {
					return crate::database::PutResult::Err(Box::new(PutError::ContentNotChanged));
				}
			} else {
				return crate::database::PutResult::Err(Box::new(PutError::GetError(
					super::GetError::Conflict {
						item_path: path.clone(),
					},
				)));
			}
		}
		Ok(crate::item::Item::Folder { .. }) => {
			if let crate::item::Item::Folder { .. } = item {
				return crate::database::PutResult::Err(Box::new(PutError::DoesNotWorksForFolders));
			} else {
				return crate::database::PutResult::Err(Box::new(PutError::GetError(
					super::GetError::Conflict {
						item_path: path.clone(),
					},
				)));
			}
		}
		Err(error) => {
			let error = *error.downcast::<super::GetError>().unwrap();

			return match error {
				super::GetError::NotFound { .. } => match item {
					crate::item::Item::Document {
						content: Some(new_content),
						content_type: new_content_type,
						..
					} => {
						let datadocument = crate::item::DataDocument {
							content_type: new_content_type,
							..Default::default()
						};

						if storage
							.set_item(
								&format!(
									"{}/{}.{}.itemdata.json",
									prefix,
									path.parent().unwrap(),
									path.file_name()
								),
								&serde_json::to_string(&datadocument).unwrap(),
							)
							.is_err()
						{
							return crate::database::PutResult::Err(Box::new(PutError::GetError(
								super::GetError::CanNotGetStorage,
							)));
						}

						if storage
							.set_item(
								&format!("{}/{}", prefix, path),
								&base64::encode(&new_content),
							)
							.is_err()
						{
							return crate::database::PutResult::Err(Box::new(PutError::GetError(
								super::GetError::CanNotGetStorage,
							)));
						}

						for ancestor in path
							.ancestors()
							.into_iter()
							.take(path.ancestors().len().saturating_sub(1))
						{
							match storage.get_item(&format!(
								"{}/{}.{}.itemdata.json",
								prefix,
								ancestor
									.parent()
									.unwrap_or_else(|| crate::item::ItemPath::from("")),
								ancestor.file_name()
							)) {
								Ok(Some(_)) => {
									return crate::database::PutResult::Err(Box::new(
										PutError::GetError(super::GetError::Conflict {
											item_path: ancestor,
										}),
									))
								}
								Ok(None) => {
									let datafolder = crate::item::DataFolder::default();

									if storage
										.set_item(
											&format!(
												"{}/{}.folder.itemdata.json",
												prefix, ancestor,
											),
											&serde_json::to_string(&datafolder).unwrap(),
										)
										.is_err()
									{
										return crate::database::PutResult::Err(Box::new(
											PutError::GetError(super::GetError::CanNotGetStorage),
										));
									}
								}
								Err(_) => {
									return crate::database::PutResult::Err(Box::new(
										PutError::CanNotDeserializeFile {
											item_path: ancestor,
											error: String::new(),
										},
									));
								}
							}
						}

						crate::database::PutResult::Created(
							datadocument.etag,
							datadocument
								.last_modified
								.unwrap_or_else(time::OffsetDateTime::now_utc),
						)
					}
					crate::item::Item::Document { content: None, .. } => {
						crate::database::PutResult::Err(Box::new(PutError::NoContentInside {
							item_path: path.clone(), // TODO : not really path, but the `item` parameter of this method
						}))
					}
					crate::item::Item::Folder { .. } => {
						crate::database::PutResult::Err(Box::new(PutError::DoesNotWorksForFolders))
					}
				},
				_ => crate::database::PutResult::Err(Box::new(PutError::GetError(error))),
			};
		}
	}
}
