pub fn build_folders(
	content: &mut std::collections::HashMap<String, Box<crate::Item>>,
	path: &mut dyn std::iter::Iterator<Item = &str>,
	cumulated_path: &str,
	tx: std::sync::mpsc::Sender<crate::database::Event>,
) -> Result<(), FolderBuildError> {
	return match path.next() {
		Some(needed) => {
			if needed.trim().is_empty() {
				Err(FolderBuildError::WrongFolderName)
			} else {
				match content.get_mut(needed) {
					Some(item) => match &mut **item {
						crate::Item::Folder {
							content: folder_content,
							..
						} => super::utils::build_folders(
							folder_content,
							path,
							&format!("{}/{}", cumulated_path, needed),
							tx,
						),
						crate::Item::Document { .. } => {
							Err(FolderBuildError::FolderDocumentConflict)
						}
					},
					None => {
						let mut child_content = std::collections::HashMap::new();

						let res = super::utils::build_folders(
							&mut child_content,
							path,
							&format!(
								"{}{}",
								if cumulated_path.ends_with('/') {
									String::from(cumulated_path)
								} else {
									format!("{}/", cumulated_path)
								},
								if needed.ends_with('/') {
									String::from(needed)
								} else {
									format!("{}/", needed)
								}
							),
							tx.clone(),
						);

						let new_item = crate::Item::Folder {
							etag: ulid::Ulid::new().to_string(),
							content: child_content,
						};

						content.insert(String::from(needed), Box::new(new_item.clone()));

						match tx.send(crate::database::Event::Create {
							path: String::from(cumulated_path),
							item: new_item.clone(),
						}) {
							Ok(()) => res,
							Err(e) => {
								Err(FolderBuildError::CanNotSendEvent(e, new_item.get_etag()))
							}
						}
					}
				}
			}
		}
		None => Ok(()),
	};
}

#[derive(Debug)]
pub enum FolderBuildError {
	FolderDocumentConflict,
	WrongFolderName,
	CanNotSendEvent(std::sync::mpsc::SendError<crate::database::Event>, String),
}

pub fn update_folders_etags(
	folder: &mut crate::Item,
	path: &mut dyn std::iter::Iterator<Item = &str>,
) -> Result<(), UpdateFoldersEtagsError> {
	let next = path.next();

	return match folder {
		crate::Item::Folder {
			etag: folder_etag,
			content: folder_content,
		} => {
			*folder_etag = ulid::Ulid::new().to_string();

			match next {
				Some(needed) => {
					if needed.trim().is_empty() {
						Err(UpdateFoldersEtagsError::WrongFolderName)
					} else {
						match folder_content.get_mut(needed) {
							Some(item) => super::utils::update_folders_etags(&mut **item, path),
							None => Err(UpdateFoldersEtagsError::MissingFolder),
						}
					}
				}
				None => Ok(()),
			}
		}
		crate::Item::Document { .. } => match next {
			Some(_) => Err(UpdateFoldersEtagsError::FolderDocumentConflict),
			None => Ok(()),
		},
	};
}

#[derive(Debug)]
pub enum UpdateFoldersEtagsError {
	FolderDocumentConflict,
	WrongFolderName,
	MissingFolder,
}

pub mod path {
	pub fn is_ok(path: &str, is_last: bool) -> bool {
		return match path {
			"" => is_last,
			"." => false,
			".." => false,
			_ => !path.contains('\0'),
		};
	}

	#[test]
	fn pfuh8x4mntyi3ej() {
		let input = "gq7tib";
		assert_eq!(is_ok(input, true), true);
		assert_eq!(is_ok(input, false), true);
	}

	#[test]
	fn b2auwz1qizhfkrolm() {
		let input = "";
		assert_eq!(is_ok(input, true), true);
		assert_eq!(is_ok(input, false), false);
	}

	#[test]
	fn hf1atgq7tibjv22p2whyhrl() {
		let input = "gq7t\0ib";
		assert_eq!(is_ok(input, true), false);
		assert_eq!(is_ok(input, false), false);
	}
}
