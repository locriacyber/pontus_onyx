pub fn build_folders(
	content: &mut std::collections::HashMap<String, Box<crate::Item>>,
	path: &mut dyn std::iter::Iterator<Item = &str>,
) -> Result<(), FolderBuildError> {
	return match path.next() {
		Some(needed) => {
			if needed.trim().is_empty() {
				Err(FolderBuildError::WrongFolderName)
			} else {
				match content.get_mut(needed) {
					Some(item) => match &mut **item {
						crate::Item::Folder {
							etag: _,
							content: folder_content,
						} => super::utils::build_folders(folder_content, path),
						crate::Item::Document {
							etag: _,
							content: _,
							content_type: _,
							last_modified: _,
						} => Err(FolderBuildError::FolderDocumentConflict),
					},
					None => {
						let mut child_content = std::collections::HashMap::new();

						let res = super::utils::build_folders(&mut child_content, path);

						content.insert(
							String::from(needed),
							Box::new(crate::Item::Folder {
								etag: ulid::Ulid::new().to_string(),
								content: child_content,
							}),
						);

						res
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
		crate::Item::Document {
			etag: _,
			content: _,
			content_type: _,
			last_modified: _,
		} => match next {
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
