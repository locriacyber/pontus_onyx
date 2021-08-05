pub fn delete(
	root_folder_path: &std::path::Path,
	path: &crate::ItemPath,
	if_match: &crate::Etag,
) -> Result<crate::Etag, Box<dyn std::error::Error>> {
	if path.is_folder() {
		return Err(Box::new(DeleteError::DoesNotWorksForFolders));
	}

	match super::get::get(root_folder_path, path, if_match, &[], false) {
		Ok(target_item) => {
			let old_target_item = target_item.get_etag().clone();

			let target_file_path = root_folder_path
				.join(&format!("{}", path.parent().unwrap()))
				.join(format!(".{}.itemdata.toml", path.file_name()));
			if target_file_path.exists() {
				if let Err(error) = std::fs::remove_file(&target_file_path) {
					return Err(Box::new(DeleteError::CanNotDelete {
						os_path: target_file_path,
						error: format!("{}", error),
					}));
				}
			}

			match std::fs::remove_file(root_folder_path.join(std::path::PathBuf::from(path))) {
				Ok(()) => {
					for parent in path
						.ancestors()
						.into_iter()
						.take(path.ancestors().len().saturating_sub(1))
						.rev()
					{
						if std::fs::read_dir(
							&root_folder_path.join(std::path::PathBuf::from(&parent)),
						)
						.unwrap()
						.filter(|e| e.as_ref().unwrap().file_name() != ".folder.itemdata.toml")
						.count() == 0
						{
							if root_folder_path
								.join(std::path::PathBuf::from(&parent))
								.join(".folder.itemdata.toml")
								.exists()
							{
								if let Err(error) = std::fs::remove_file(
									root_folder_path
										.join(std::path::PathBuf::from(&parent))
										.join(".folder.itemdata.toml"),
								) {
									return Err(Box::new(DeleteError::CanNotDelete {
										os_path: root_folder_path
											.join(std::path::PathBuf::from(&parent))
											.join(".folder.itemdata.toml"),
										error: format!("{}", error),
									}));
								}
							}

							if root_folder_path
								.join(std::path::PathBuf::from(&parent))
								.exists()
							{
								if let Err(error) = std::fs::remove_dir(
									root_folder_path.join(std::path::PathBuf::from(&parent)),
								) {
									return Err(Box::new(DeleteError::CanNotDelete {
										os_path: root_folder_path
											.join(std::path::PathBuf::from(&parent)),
										error: format!("{}", error),
									}));
								}
							}
						} else {
							let mut folderdata = match std::fs::read(
								root_folder_path
									.join(std::path::PathBuf::from(&parent))
									.join(".folder.itemdata.toml"),
							) {
								Ok(folderdata_content) => {
									match toml::from_slice::<crate::DataFolder>(&folderdata_content)
									{
										Ok(res) => res,
										Err(error) => {
											return Err(Box::new(
												DeleteError::CanNotDeserializeFile {
													os_path: root_folder_path
														.join(std::path::PathBuf::from(&parent)),
													error: format!("{}", error),
												},
											));
										}
									}
								}
								Err(error) => {
									return Err(Box::new(DeleteError::CanNotReadFile {
										os_path: root_folder_path
											.join(std::path::PathBuf::from(&parent))
											.join(".folder.itemdata.toml"),
										error: format!("{}", error),
									}));
								}
							};

							folderdata.datastruct_version = String::from(env!("CARGO_PKG_VERSION"));
							folderdata.etag = crate::Etag::new();

							match toml::to_vec(&folderdata) {
								Ok(folderdata_content) => {
									if let Err(error) = std::fs::write(
										root_folder_path
											.join(std::path::PathBuf::from(&parent))
											.join(".folder.itemdata.toml"),
										&folderdata_content,
									) {
										return Err(Box::new(DeleteError::CanNotWriteFile {
											os_path: root_folder_path
												.join(std::path::PathBuf::from(&parent))
												.join(".folder.itemdata.toml"),
											error: format!("{}", error),
										}));
									}
								}
								Err(error) => {
									return Err(Box::new(DeleteError::CanNotSerializeFile {
										os_path: root_folder_path
											.join(std::path::PathBuf::from(&parent))
											.join(".folder.itemdata.toml"),
										error: format!("{}", error),
									}));
								}
							}
						}
					}

					return Ok(old_target_item);
				}
				Err(error) => {
					return Err(Box::new(DeleteError::CanNotDelete {
						os_path: root_folder_path.join(std::path::PathBuf::from(path)),
						error: format!("{}", error),
					}));
				}
			}
		}
		Err(boxed_error) => {
			return Err(Box::new(DeleteError::GetError(
				*boxed_error.downcast::<super::GetError>().unwrap(),
			)));
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeleteError {
	GetError(super::GetError),
	DoesNotWorksForFolders,
	CanNotDelete {
		os_path: std::path::PathBuf,
		error: String,
	},
	CanNotReadFile {
		os_path: std::path::PathBuf,
		error: String,
	},
	CanNotWriteFile {
		os_path: std::path::PathBuf,
		error: String,
	},
	CanNotSerializeFile {
		os_path: std::path::PathBuf,
		error: String,
	},
	CanNotDeserializeFile {
		os_path: std::path::PathBuf,
		error: String,
	},
}
impl std::fmt::Display for DeleteError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::GetError(get_error) => std::fmt::Display::fmt(get_error, f),
			Self::DoesNotWorksForFolders => f.write_str("this method does not works for folders"),
			Self::CanNotDelete { os_path, error } => f.write_fmt(format_args!(
				"can not delete file `{:?}` because : {}",
				os_path, error
			)),
			Self::CanNotReadFile { os_path, error } => f.write_fmt(format_args!(
				"can not read file `{:?}` because : {}",
				os_path, error
			)),
			Self::CanNotWriteFile { os_path, error } => f.write_fmt(format_args!(
				"can not write file `{:?}` because : {}",
				os_path, error
			)),
			Self::CanNotSerializeFile { os_path, error } => f.write_fmt(format_args!(
				"can not serialize file `{:?}` because : {}",
				os_path, error
			)),
			Self::CanNotDeserializeFile { os_path, error } => f.write_fmt(format_args!(
				"can not deserialize file `{:?}` because : {}",
				os_path, error
			)),
		}
	}
}
impl std::error::Error for DeleteError {}
#[cfg(feature = "server_bin")]
impl crate::database::Error for DeleteError {
	fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse {
		match self {
			// TODO : we have to find a way to change method
			Self::GetError(get_error) => {
				crate::database::Error::to_response(get_error, origin, should_have_body)
			}
			Self::DoesNotWorksForFolders => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::CanNotDelete {
				os_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
			Self::CanNotReadFile {
				os_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
			Self::CanNotWriteFile {
				os_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
			Self::CanNotSerializeFile {
				os_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
			Self::CanNotDeserializeFile {
				os_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				should_have_body,
			),
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(non_snake_case)]

	use std::convert::TryFrom;

	use super::{super::GetError, delete, DeleteError};
	use crate::{Etag, Item, ItemPath};

	// TODO : test last_modified

	fn build_test_db() -> (
		tempfile::TempDir,
		Etag,
		Etag,
		Etag,
		Etag,
		Etag,
		Etag,
		Etag,
		Etag,
	) {
		let AAA = Item::new_doc(b"AAA", "text/plain");
		let AA = Item::new_folder(vec![("AAA", AAA.clone())]);
		let AB = Item::new_doc(b"AB", "text/plain");
		let A = Item::new_folder(vec![("AA", AA.clone()), ("AB", AB.clone())]);

		let BA = Item::new_doc(b"BA", "text/plain");
		let B = Item::new_folder(vec![("BA", BA.clone())]);
		let public = Item::new_folder(vec![("B", B.clone())]);

		let root = Item::new_folder(vec![("A", A.clone()), ("public", public.clone())]);

		////////////////////////////////////////////////////////////////////////////////////////////////

		let tmp_folder = tempfile::tempdir().unwrap();
		println!(
			"folder dedicated to this test : {}",
			tmp_folder.path().to_string_lossy()
		);

		let tmp_folder_path = tmp_folder.path().to_path_buf();

		let A_path = tmp_folder_path.join("A");
		let AA_path = A_path.join("AA");
		let AB_path = A_path.join("AB");
		let AAA_path = AA_path.join("AAA");
		let public_path = tmp_folder_path.join("public");
		let B_path = public_path.join("B");
		let BA_path = B_path.join("BA");

		std::fs::create_dir_all(&AA_path).unwrap();
		std::fs::create_dir_all(&B_path).unwrap();

		let root_data_path = tmp_folder_path.join(".folder.itemdata.toml");
		std::fs::write(
			&root_data_path,
			toml::to_string(&crate::DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: root.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

		let A_data_path = A_path.join(".folder.itemdata.toml");
		std::fs::write(
			A_data_path,
			toml::to_string(&crate::DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: A.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

		let AA_data_path = AA_path.join(".folder.itemdata.toml");
		std::fs::write(
			AA_data_path,
			toml::to_string(&crate::DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: AA.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

		let AB_data_path = A_path.join(".AB.itemdata.toml");
		std::fs::write(
			AB_data_path,
			toml::to_string(&crate::DataDocument::try_from(AB.clone()).unwrap()).unwrap(),
		)
		.unwrap();

		if let Item::Document {
			content: Some(content),
			..
		} = &AB
		{
			std::fs::write(AB_path, content).unwrap();
		} else {
			panic!()
		}

		let AAA_data_path = AA_path.join(".AAA.itemdata.toml");
		std::fs::write(
			AAA_data_path,
			toml::to_string(&crate::DataDocument::try_from(AAA.clone()).unwrap()).unwrap(),
		)
		.unwrap();

		if let Item::Document {
			content: Some(content),
			..
		} = &AAA
		{
			std::fs::write(AAA_path, content).unwrap();
		} else {
			panic!()
		}

		let public_data_path = public_path.join(".folder.itemdata.toml");
		std::fs::write(
			public_data_path,
			toml::to_string(&crate::DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: public.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

		let B_data_path = B_path.join(".folder.itemdata.toml");
		std::fs::write(
			B_data_path,
			toml::to_string(&crate::DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: B.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

		let BA_data_path = B_path.join(".BA.itemdata.toml");
		std::fs::write(
			BA_data_path,
			toml::to_string(&crate::DataDocument::try_from(BA.clone()).unwrap()).unwrap(),
		)
		.unwrap();

		if let Item::Document {
			content: Some(content),
			..
		} = &BA
		{
			std::fs::write(BA_path, content).unwrap();
		} else {
			panic!()
		}

		return (
			tmp_folder,
			root.get_etag().clone(),
			A.get_etag().clone(),
			AA.get_etag().clone(),
			AB.get_etag().clone(),
			AAA.get_etag().clone(),
			public.get_etag().clone(),
			B.get_etag().clone(),
			BA.get_etag().clone(),
		);
	}

	#[allow(dead_code)]
	fn debug_copy(tmp_folder_path: &std::path::Path, copy_target: &std::path::Path) {
		fs_extra::dir::copy(
			tmp_folder_path,
			copy_target,
			&fs_extra::dir::CopyOptions {
				overwrite: true,
				skip_exist: false,
				copy_inside: true,
				content_only: false,
				..fs_extra::dir::CopyOptions::new()
			},
		)
		.unwrap();
	}

	#[test]
	fn simple_delete_on_not_existing() {
		let tmp_folder = tempfile::tempdir().unwrap();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		assert_eq!(
			*delete(
				&tmp_folder_path,
				&ItemPath::from("A/AA/AAA"),
				&Etag::from(""),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::GetError(GetError::NotFound {
				item_path: ItemPath::from("A/")
			})
		);

		assert_eq!(std::fs::read_dir(&tmp_folder_path).unwrap().count(), 0);
	}

	#[test]
	fn simple_delete_on_existing() {
		let (tmp_folder, root_etag, A_etag, _, AB_etag, AAA_etag, public_etag, B_etag, BA_etag) =
			build_test_db();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		let old_AAA_etag = delete(
			&tmp_folder_path,
			&ItemPath::from("A/AA/AAA"),
			&Etag::from(""),
		)
		.unwrap();

		assert_eq!(AAA_etag, old_AAA_etag);

		assert!(tmp_folder_path.exists());
		assert!(tmp_folder_path.join(".folder.itemdata.toml").exists());
		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_ne!(root_datafile.etag, root_etag);

		assert!(tmp_folder_path.join("A").exists());
		assert!(tmp_folder_path
			.join("A")
			.join(".folder.itemdata.toml")
			.exists());
		let A_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_ne!(A_datafile.etag, A_etag);

		assert!(!tmp_folder_path.join("A").join("AA").exists());
		assert!(!tmp_folder_path
			.join("A")
			.join("AA")
			.join(".folder.itemdata.toml")
			.exists());

		assert!(!tmp_folder_path.join("A").join("AA").join("AAA").exists());
		assert!(!tmp_folder_path
			.join("A")
			.join("AA")
			.join(".AAA.itemdata.toml")
			.exists());

		assert!(tmp_folder_path.join("A").join("AB").exists());
		assert!(tmp_folder_path.join("A").join(".AB.itemdata.toml").exists());
		let AB_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(AB_datafile.etag, AB_etag);

		assert!(tmp_folder_path.join("public").exists());
		assert!(tmp_folder_path
			.join("public")
			.join(".folder.itemdata.toml")
			.exists());
		let public_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("public").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(public_datafile.etag, public_etag);

		assert!(tmp_folder_path.join("public").join("B").exists());
		assert!(tmp_folder_path
			.join("public")
			.join("B")
			.join(".folder.itemdata.toml")
			.exists());
		let B_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("public")
					.join("B")
					.join(".folder.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(B_datafile.etag, B_etag);

		assert!(tmp_folder_path.join("public").join("B").join("BA").exists());
		assert!(tmp_folder_path
			.join("public")
			.join("B")
			.join(".BA.itemdata.toml")
			.exists());
		let BA_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("public")
					.join("B")
					.join(".BA.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(BA_datafile.etag, BA_etag);
	}

	#[test]
	fn does_not_works_for_folders() {
		let (
			tmp_folder,
			root_etag,
			A_etag,
			AA_etag,
			AB_etag,
			AAA_etag,
			public_etag,
			B_etag,
			BA_etag,
		) = build_test_db();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		assert_eq!(
			*delete(&tmp_folder_path, &ItemPath::from("A/AA/"), &Etag::from(""),)
				.unwrap_err()
				.downcast::<DeleteError>()
				.unwrap(),
			DeleteError::DoesNotWorksForFolders,
		);

		assert!(tmp_folder_path.exists());
		assert!(tmp_folder_path.join(".folder.itemdata.toml").exists());
		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(root_datafile.etag, root_etag);

		assert!(tmp_folder_path.join("A").exists());
		assert!(tmp_folder_path
			.join("A")
			.join(".folder.itemdata.toml")
			.exists());
		let A_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(A_datafile.etag, A_etag);

		assert!(tmp_folder_path.join("A").join("AA").exists());
		assert!(tmp_folder_path
			.join("A")
			.join("AA")
			.join(".folder.itemdata.toml")
			.exists());
		let AA_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("A")
					.join("AA")
					.join(".folder.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(AA_datafile.etag, AA_etag);

		assert!(tmp_folder_path.join("A").join("AA").join("AAA").exists());
		assert!(tmp_folder_path
			.join("A")
			.join("AA")
			.join(".AAA.itemdata.toml")
			.exists());
		let AAA_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("A")
					.join("AA")
					.join(".AAA.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(AAA_datafile.etag, AAA_etag);

		assert!(tmp_folder_path.join("A").join("AB").exists());
		assert!(tmp_folder_path.join("A").join(".AB.itemdata.toml").exists());
		let AB_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(AB_datafile.etag, AB_etag);

		assert!(tmp_folder_path.join("public").exists());
		assert!(tmp_folder_path
			.join("public")
			.join(".folder.itemdata.toml")
			.exists());
		let public_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("public").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(public_datafile.etag, public_etag);

		assert!(tmp_folder_path.join("public").join("B").exists());
		assert!(tmp_folder_path
			.join("public")
			.join("B")
			.join(".folder.itemdata.toml")
			.exists());
		let B_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("public")
					.join("B")
					.join(".folder.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(B_datafile.etag, B_etag);

		assert!(tmp_folder_path.join("public").join("B").join("BA").exists());
		assert!(tmp_folder_path
			.join("public")
			.join("B")
			.join(".BA.itemdata.toml")
			.exists());
		let BA_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("public")
					.join("B")
					.join(".BA.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(BA_datafile.etag, BA_etag);
	}

	#[test]
	fn delete_with_if_match_not_found() {
		let (
			tmp_folder,
			root_etag,
			A_etag,
			AA_etag,
			AB_etag,
			AAA_etag,
			public_etag,
			B_etag,
			BA_etag,
		) = build_test_db();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		assert_eq!(
			*delete(
				&tmp_folder_path,
				&ItemPath::from("A/AA/AAA"),
				&Etag::from("OTHER_ETAG"),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::GetError(GetError::NoIfMatch {
				item_path: ItemPath::from("A/AA/AAA"),
				found: AAA_etag.clone(),
				search: Etag::from("OTHER_ETAG")
			})
		);

		assert!(tmp_folder_path.exists());
		assert!(tmp_folder_path.join(".folder.itemdata.toml").exists());
		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(root_datafile.etag, root_etag);

		assert!(tmp_folder_path.join("A").exists());
		assert!(tmp_folder_path
			.join("A")
			.join(".folder.itemdata.toml")
			.exists());
		let A_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(A_datafile.etag, A_etag);

		assert!(tmp_folder_path.join("A").join("AA").exists());
		assert!(tmp_folder_path
			.join("A")
			.join("AA")
			.join(".folder.itemdata.toml")
			.exists());
		let AA_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("A")
					.join("AA")
					.join(".folder.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(AA_datafile.etag, AA_etag);

		assert!(tmp_folder_path.join("A").join("AA").join("AAA").exists());
		assert!(tmp_folder_path
			.join("A")
			.join("AA")
			.join(".AAA.itemdata.toml")
			.exists());
		let AAA_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("A")
					.join("AA")
					.join(".AAA.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(AAA_datafile.etag, AAA_etag);

		assert!(tmp_folder_path.join("A").join("AB").exists());
		assert!(tmp_folder_path.join("A").join(".AB.itemdata.toml").exists());
		let AB_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(AB_datafile.etag, AB_etag);

		assert!(tmp_folder_path.join("public").exists());
		assert!(tmp_folder_path
			.join("public")
			.join(".folder.itemdata.toml")
			.exists());
		let public_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("public").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(public_datafile.etag, public_etag);

		assert!(tmp_folder_path.join("public").join("B").exists());
		assert!(tmp_folder_path
			.join("public")
			.join("B")
			.join(".folder.itemdata.toml")
			.exists());
		let B_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("public")
					.join("B")
					.join(".folder.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(B_datafile.etag, B_etag);

		assert!(tmp_folder_path.join("public").join("B").join("BA").exists());
		assert!(tmp_folder_path
			.join("public")
			.join("B")
			.join(".BA.itemdata.toml")
			.exists());
		let BA_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("public")
					.join("B")
					.join(".BA.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(BA_datafile.etag, BA_etag);
	}

	#[test]
	fn delete_with_if_match_found() {
		let (tmp_folder, root_etag, A_etag, _, AB_etag, AAA_etag, public_etag, B_etag, BA_etag) =
			build_test_db();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		let old_AAA_etag =
			delete(&tmp_folder_path, &ItemPath::from("A/AA/AAA"), &AAA_etag).unwrap();

		assert_eq!(old_AAA_etag, AAA_etag);

		assert!(tmp_folder_path.exists());
		assert!(tmp_folder_path.join(".folder.itemdata.toml").exists());
		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_ne!(root_datafile.etag, root_etag);

		assert!(tmp_folder_path.join("A").exists());
		assert!(tmp_folder_path
			.join("A")
			.join(".folder.itemdata.toml")
			.exists());
		let A_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_ne!(A_datafile.etag, A_etag);

		assert!(!tmp_folder_path.join("A").join("AA").exists());
		assert!(!tmp_folder_path
			.join("A")
			.join("AA")
			.join(".folder.itemdata.toml")
			.exists());

		assert!(!tmp_folder_path.join("A").join("AA").join("AAA").exists());
		assert!(!tmp_folder_path
			.join("A")
			.join("AA")
			.join(".AAA.itemdata.toml")
			.exists());

		assert!(tmp_folder_path.join("A").join("AB").exists());
		assert!(tmp_folder_path.join("A").join(".AB.itemdata.toml").exists());
		let AB_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(AB_datafile.etag, AB_etag);

		assert!(tmp_folder_path.join("public").exists());
		assert!(tmp_folder_path
			.join("public")
			.join(".folder.itemdata.toml")
			.exists());
		let public_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("public").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(public_datafile.etag, public_etag);

		assert!(tmp_folder_path.join("public").join("B").exists());
		assert!(tmp_folder_path
			.join("public")
			.join("B")
			.join(".folder.itemdata.toml")
			.exists());
		let B_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("public")
					.join("B")
					.join(".folder.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(B_datafile.etag, B_etag);

		assert!(tmp_folder_path.join("public").join("B").join("BA").exists());
		assert!(tmp_folder_path
			.join("public")
			.join("B")
			.join(".BA.itemdata.toml")
			.exists());
		let BA_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("public")
					.join("B")
					.join(".BA.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(BA_datafile.etag, BA_etag);
	}

	#[test]
	fn delete_with_if_match_all() {
		let (tmp_folder, root_etag, A_etag, _, AB_etag, AAA_etag, public_etag, B_etag, BA_etag) =
			build_test_db();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		let old_AAA_etag = delete(
			&tmp_folder_path,
			&ItemPath::from("A/AA/AAA"),
			&Etag::from("*"),
		)
		.unwrap();

		assert_eq!(old_AAA_etag, AAA_etag);

		assert!(tmp_folder_path.exists());
		assert!(tmp_folder_path.join(".folder.itemdata.toml").exists());
		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_ne!(root_datafile.etag, root_etag);

		assert!(tmp_folder_path.join("A").exists());
		assert!(tmp_folder_path
			.join("A")
			.join(".folder.itemdata.toml")
			.exists());
		let A_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_ne!(A_datafile.etag, A_etag);

		assert!(!tmp_folder_path.join("A").join("AA").exists());
		assert!(!tmp_folder_path
			.join("A")
			.join("AA")
			.join(".folder.itemdata.toml")
			.exists());

		assert!(!tmp_folder_path.join("A").join("AA").join("AAA").exists());
		assert!(!tmp_folder_path
			.join("A")
			.join("AA")
			.join(".AAA.itemdata.toml")
			.exists());

		assert!(tmp_folder_path.join("A").join("AB").exists());
		assert!(tmp_folder_path.join("A").join(".AB.itemdata.toml").exists());
		let AB_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(AB_datafile.etag, AB_etag);

		assert!(tmp_folder_path.join("public").exists());
		assert!(tmp_folder_path
			.join("public")
			.join(".folder.itemdata.toml")
			.exists());
		let public_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("public").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(public_datafile.etag, public_etag);

		assert!(tmp_folder_path.join("public").join("B").exists());
		assert!(tmp_folder_path
			.join("public")
			.join("B")
			.join(".folder.itemdata.toml")
			.exists());
		let B_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("public")
					.join("B")
					.join(".folder.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(B_datafile.etag, B_etag);

		assert!(tmp_folder_path.join("public").join("B").join("BA").exists());
		assert!(tmp_folder_path
			.join("public")
			.join("B")
			.join(".BA.itemdata.toml")
			.exists());
		let BA_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("public")
					.join("B")
					.join(".BA.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(BA_datafile.etag, BA_etag);
	}

	#[test]
	fn delete_with_existing_folder_conflict() {
		let (
			tmp_folder,
			root_etag,
			A_etag,
			AA_etag,
			AB_etag,
			AAA_etag,
			public_etag,
			B_etag,
			BA_etag,
		) = build_test_db();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		assert_eq!(
			*delete(&tmp_folder_path, &ItemPath::from("A/AA"), &Etag::from(""),)
				.unwrap_err()
				.downcast::<DeleteError>()
				.unwrap(),
			DeleteError::GetError(GetError::Conflict {
				item_path: ItemPath::from("A/AA/")
			})
		);

		assert!(tmp_folder_path.exists());
		assert!(tmp_folder_path.join(".folder.itemdata.toml").exists());
		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(root_datafile.etag, root_etag);

		assert!(tmp_folder_path.join("A").exists());
		assert!(tmp_folder_path
			.join("A")
			.join(".folder.itemdata.toml")
			.exists());
		let A_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(A_datafile.etag, A_etag);

		assert!(tmp_folder_path.join("A").join("AA").exists());
		assert!(tmp_folder_path
			.join("A")
			.join("AA")
			.join(".folder.itemdata.toml")
			.exists());
		let AA_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("A")
					.join("AA")
					.join(".folder.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(AA_datafile.etag, AA_etag);

		assert!(tmp_folder_path.join("A").join("AA").join("AAA").exists());
		assert!(tmp_folder_path
			.join("A")
			.join("AA")
			.join(".AAA.itemdata.toml")
			.exists());
		let AAA_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("A")
					.join("AA")
					.join(".AAA.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(AAA_datafile.etag, AAA_etag);

		assert!(tmp_folder_path.join("A").join("AB").exists());
		assert!(tmp_folder_path.join("A").join(".AB.itemdata.toml").exists());
		let AB_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(AB_datafile.etag, AB_etag);

		assert!(tmp_folder_path.join("public").exists());
		assert!(tmp_folder_path
			.join("public")
			.join(".folder.itemdata.toml")
			.exists());
		let public_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("public").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(public_datafile.etag, public_etag);

		assert!(tmp_folder_path.join("public").join("B").exists());
		assert!(tmp_folder_path
			.join("public")
			.join("B")
			.join(".folder.itemdata.toml")
			.exists());
		let B_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("public")
					.join("B")
					.join(".folder.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(B_datafile.etag, B_etag);

		assert!(tmp_folder_path.join("public").join("B").join("BA").exists());
		assert!(tmp_folder_path
			.join("public")
			.join("B")
			.join(".BA.itemdata.toml")
			.exists());
		let BA_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("public")
					.join("B")
					.join(".BA.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(BA_datafile.etag, BA_etag);
	}

	#[test]
	fn delete_in_public() {
		let (tmp_folder, root_etag, A_etag, AA_etag, AB_etag, AAA_etag, _, _, BA_etag) =
			build_test_db();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		let old_BA_etag = delete(
			&tmp_folder_path,
			&ItemPath::from("public/B/BA"),
			&Etag::from(""),
		)
		.unwrap();

		assert_eq!(old_BA_etag, BA_etag);

		assert!(tmp_folder_path.exists());
		assert!(tmp_folder_path.join(".folder.itemdata.toml").exists());
		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_ne!(root_datafile.etag, root_etag);

		assert!(tmp_folder_path.join("A").exists());
		assert!(tmp_folder_path
			.join("A")
			.join(".folder.itemdata.toml")
			.exists());
		let A_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(A_datafile.etag, A_etag);

		assert!(tmp_folder_path.join("A").join("AA").exists());
		assert!(tmp_folder_path
			.join("A")
			.join("AA")
			.join(".folder.itemdata.toml")
			.exists());
		let AA_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("A")
					.join("AA")
					.join(".folder.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(AA_datafile.etag, AA_etag);

		assert!(tmp_folder_path.join("A").join("AA").join("AAA").exists());
		assert!(tmp_folder_path
			.join("A")
			.join("AA")
			.join(".AAA.itemdata.toml")
			.exists());
		let AAA_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("A")
					.join("AA")
					.join(".AAA.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(AAA_datafile.etag, AAA_etag);

		assert!(tmp_folder_path.join("A").join("AB").exists());
		assert!(tmp_folder_path.join("A").join(".AB.itemdata.toml").exists());
		let AB_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(AB_datafile.etag, AB_etag);

		assert!(!tmp_folder_path.join("public").exists());
		assert!(!tmp_folder_path
			.join("public")
			.join(".folder.itemdata.toml")
			.exists());

		assert!(!tmp_folder_path.join("public").join("B").exists());
		assert!(!tmp_folder_path
			.join("public")
			.join("B")
			.join(".folder.itemdata.toml")
			.exists());

		assert!(!tmp_folder_path.join("public").join("B").join("BA").exists());
		assert!(!tmp_folder_path
			.join("public")
			.join("B")
			.join(".BA.itemdata.toml")
			.exists());
	}

	#[test]
	fn delete_in_incorrect_path() {
		let tmp_folder = tempfile::tempdir().unwrap();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		assert_eq!(
			*delete(&tmp_folder_path, &ItemPath::from("A/A\0A"), &Etag::from(""),)
				.unwrap_err()
				.downcast::<DeleteError>()
				.unwrap(),
			DeleteError::GetError(GetError::IncorrectItemName {
				item_path: ItemPath::from("A/A\0A"),
				error: String::from("`A\0A` should not contains `\\0` character")
			})
		);

		assert_eq!(std::fs::read_dir(&tmp_folder_path).unwrap().count(), 0);
	}
}
