impl super::Database {
	pub fn delete(&mut self, path: &str, if_match: &str) -> Result<String, ErrorDelete> {
		match self.get(path, if_match, vec![]) {
			Ok(crate::Item::Document { .. }) => {
				/*
				TODO : option to keep old documents ?
					A provider MAY offer version rollback functionality to its users,
					but this specification does not define the interface for that.
				*/
				let paths: Vec<&str> = path.split('/').collect();

				if paths
					.iter()
					.enumerate()
					.all(|(i, &e)| super::utils::path::is_ok(e, i == (paths.len() - 1)))
				{
					if paths.last().unwrap() != &"" {
						match self.get(path, if_match, vec![]) {
							Ok(crate::Item::Document { .. }) => {
								let parent = self.fetch_item_mut(
									&paths
										.clone()
										.iter()
										.take(paths.len() - 1)
										.cloned()
										.collect::<Vec<&str>>(),
								);

								if let Ok(Some(crate::Item::Folder { content, .. })) = parent {
									match content.remove(*paths.last().unwrap()) {
										Some(old_version) => {
											match super::utils::update_folders_etags(
												&mut self.content,
												&mut paths.iter().cloned().take(paths.len() - 1),
											) {
												Ok(()) => {
													for i in 0..=paths.len() {
														self.cleanup_empty_folders(
															&paths
																.iter()
																.take(paths.len() - i)
																.fold(String::new(), |acc, e| {
																	acc + *e + "/"
																}),
														)
														.ok(); // errors are not important here
													}

													match self.changes_tx.send(
														crate::database::Event::Delete {
															path: String::from(path),
														},
													) {
														Ok(()) => Ok(old_version.get_etag()),
														Err(e) => {
															Err(ErrorDelete::CanNotSendEvent(
																e,
																old_version.get_etag(),
															))
														}
													}
												}
												Err(e) => {
													// TODO : is following conversion is OK ?
													Err(match e {
														super::utils::UpdateFoldersEtagsError::FolderDocumentConflict => ErrorDelete::Conflict,
														super::utils::UpdateFoldersEtagsError::MissingFolder => ErrorDelete::NotFound,
														super::utils::UpdateFoldersEtagsError::WrongFolderName => ErrorDelete::WrongPath,
													})
												}
											}
										}
										None => Err(ErrorDelete::NotFound),
									}
								} else {
									Err(ErrorDelete::NotFound)
								}
							}
							Ok(crate::Item::Folder { .. }) => {
								Err(ErrorDelete::WorksOnlyForDocument)
							}
							Err(super::get::ErrorGet::CanNotBeListed) => {
								Err(ErrorDelete::WorksOnlyForDocument)
							} // TODO : is this OK ?
							Err(super::get::ErrorGet::Conflict) => Err(ErrorDelete::Conflict),
							Err(super::get::ErrorGet::IfMatchNotFound) => {
								Err(ErrorDelete::IfMatchNotFound)
							}
							Err(super::get::ErrorGet::IfNoneMatch) => {
								Err(ErrorDelete::InternalError)
							} // should never happen
							Err(super::get::ErrorGet::NotFound) => Err(ErrorDelete::NotFound),
							Err(super::get::ErrorGet::WrongPath) => Err(ErrorDelete::WrongPath),
						}
					} else {
						Err(ErrorDelete::WorksOnlyForDocument)
					}
				} else {
					Err(ErrorDelete::WrongPath)
				}
			}
			Ok(crate::Item::Folder { .. }) => Err(ErrorDelete::WorksOnlyForDocument),
			Err(crate::database::ErrorGet::CanNotBeListed) => {
				Err(ErrorDelete::WorksOnlyForDocument)
			}
			Err(crate::database::ErrorGet::Conflict) => Err(ErrorDelete::Conflict),
			Err(crate::database::ErrorGet::IfMatchNotFound) => Err(ErrorDelete::IfMatchNotFound),
			Err(crate::database::ErrorGet::IfNoneMatch) => Err(ErrorDelete::IfNoneMatch),
			Err(crate::database::ErrorGet::NotFound) => Err(ErrorDelete::NotFound),
			Err(crate::database::ErrorGet::WrongPath) => Err(ErrorDelete::WrongPath),
		}
	}
}

mod error;
pub use error::*;
