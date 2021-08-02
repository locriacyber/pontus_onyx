pub fn put(
	root_folder_path: &std::path::Path,
	path: &std::path::Path,
	if_match: &crate::Etag,
	if_none_match: &[&crate::Etag],
	new_item: crate::Item,
) -> crate::database::PutResult {
	// TODO : test if path is document and new_item is folder (and vice-versa) ?
	if path.to_str().unwrap().ends_with('/')
		|| path.to_str().unwrap().ends_with('\\')
		|| path == std::path::PathBuf::from("")
	{
		return crate::database::PutResult::Err(Box::new(PutError::DoesNotWorksForFolders));
	}

	let item_fetch = super::get::get(root_folder_path, path, if_match, if_none_match, true);

	let target_content_path = root_folder_path.join(path);
	let target_data_path = root_folder_path
		.join(path.parent().unwrap_or_else(|| std::path::Path::new("")))
		.join(format!(
			".{}.itemdata.toml",
			path.file_name().unwrap().to_str().unwrap()
		));

	match item_fetch {
		Ok(crate::Item::Document {
			content: old_content,
			content_type: old_content_type,
			..
		}) => {
			if let crate::Item::Document {
				content: new_content,
				content_type: new_content_type,
				..
			} = new_item
			{
				if new_content != old_content || new_content_type != old_content_type {
					let new_etag = crate::Etag::new();

					for parent_path in path.ancestors().skip(1) {
						let target_parent_path = root_folder_path.join(parent_path);
						let parent_datafile_path = target_parent_path.join(".folder.itemdata.toml");

						let mut parent_datafile: crate::DataFolder = {
							let file_content = std::fs::read(&parent_datafile_path);
							match file_content {
								Ok(file_content) => match toml::from_slice(&file_content) {
									Ok(file_content) => file_content,
									Err(error) => {
										return crate::database::PutResult::Err(Box::new(
											PutError::CanNotDeserializeFile {
												path: parent_datafile_path,
												error: format!("{}", error),
											},
										));
									}
								},
								Err(error) => {
									return crate::database::PutResult::Err(Box::new(
										PutError::CanNotReadFile {
											path: parent_datafile_path,
											error: format!("{}", error),
										},
									));
								}
							}
						};

						parent_datafile.datastruct_version =
							String::from(env!("CARGO_PKG_VERSION"));
						parent_datafile.etag = crate::Etag::new();

						match toml::to_vec(&parent_datafile) {
							Ok(parent_datafile) => {
								if let Err(error) =
									std::fs::write(&parent_datafile_path, &parent_datafile)
								{
									return crate::database::PutResult::Err(Box::new(
										PutError::CanNotWriteFile {
											path: parent_datafile_path,
											error: format!("{}", error),
										},
									));
								}
							}
							Err(error) => {
								return crate::database::PutResult::Err(Box::new(
									PutError::CanNotSerializeFile {
										path: parent_datafile_path,
										error: format!("{}", error),
									},
								));
							}
						}
					}

					if let Some(new_content) = new_content {
						if let Err(error) = std::fs::write(&target_content_path, &new_content) {
							return crate::database::PutResult::Err(Box::new(
								PutError::CanNotWriteFile {
									path: target_content_path,
									error: format!("{}", error),
								},
							));
						}
					}

					match toml::to_vec(&crate::DataDocument {
						datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
						etag: new_etag.clone(),
						content_type: new_content_type,
						last_modified: chrono::Utc::now(),
					}) {
						Ok(datadoc) => {
							if let Err(error) = std::fs::write(&target_data_path, &datadoc) {
								return crate::database::PutResult::Err(Box::new(
									PutError::CanNotWriteFile {
										path: target_data_path,
										error: format!("{}", error),
									},
								));
							}
						}
						Err(error) => {
							return crate::database::PutResult::Err(Box::new(
								PutError::CanNotSerializeFile {
									path: target_data_path,
									error: format!("{}", error),
								},
							));
						}
					}

					return crate::database::PutResult::Updated(new_etag);
				} else {
					return crate::database::PutResult::Err(Box::new(PutError::ContentNotChanged));
				}
			} else {
				return crate::database::PutResult::Err(Box::new(PutError::DoesNotWorksForFolders));
			}
		}
		Ok(crate::Item::Folder { .. }) => {
			let tmp_path = path.to_str().unwrap();
			return crate::database::PutResult::Err(Box::new(super::GetError::Conflict {
				item_path: std::path::PathBuf::from(
					tmp_path.strip_suffix('/').unwrap_or(tmp_path).to_string() + "/",
				),
			}));
		}
		Err(boxed_error) => {
			let get_error = *boxed_error.downcast::<super::GetError>().unwrap();

			if let super::GetError::NotFound { item_path: _ } = get_error {
				if let crate::Item::Document {
					content: new_content,
					content_type: new_content_type,
					..
				} = new_item
				{
					let new_etag = crate::Etag::new();

					for parent_path in path.ancestors().skip(1) {
						let target_parent_path = root_folder_path.join(parent_path);
						let parent_datafile_path = target_parent_path.join(".folder.itemdata.toml");

						if let Err(error) = std::fs::create_dir_all(&target_parent_path) {
							return crate::database::PutResult::Err(Box::new(
								PutError::CanNotWriteFile {
									path: target_parent_path,
									error: format!("{}", error),
								},
							));
						}

						let parent_datafile = crate::DataFolder {
							datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
							etag: crate::Etag::new(),
						};

						match toml::to_vec(&parent_datafile) {
							Ok(datafile) => {
								if let Err(error) = std::fs::write(&parent_datafile_path, &datafile)
								{
									return crate::database::PutResult::Err(Box::new(
										PutError::CanNotWriteFile {
											path: parent_datafile_path,
											error: format!("{}", error),
										},
									));
								}
							}
							Err(error) => {
								return crate::database::PutResult::Err(Box::new(
									PutError::CanNotSerializeFile {
										path: parent_datafile_path,
										error: format!("{}", error),
									},
								));
							}
						}
					}

					if let Some(new_content) = new_content {
						if let Err(error) = std::fs::write(&target_content_path, &new_content) {
							return crate::database::PutResult::Err(Box::new(
								PutError::CanNotWriteFile {
									path: target_content_path,
									error: format!("{}", error),
								},
							));
						}
					}

					match toml::to_vec(&crate::DataDocument {
						datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
						etag: new_etag.clone(),
						content_type: new_content_type,
						last_modified: chrono::Utc::now(),
					}) {
						Ok(datafile) => {
							if let Err(error) = std::fs::write(&target_data_path, &datafile) {
								return crate::database::PutResult::Err(Box::new(
									PutError::CanNotWriteFile {
										path: target_data_path,
										error: format!("{}", error),
									},
								));
							}
						}
						Err(error) => {
							return crate::database::PutResult::Err(Box::new(
								PutError::CanNotSerializeFile {
									path: target_data_path,
									error: format!("{}", error),
								},
							));
						}
					}

					return crate::database::PutResult::Updated(new_etag);
				} else {
					return crate::database::PutResult::Err(Box::new(
						PutError::DoesNotWorksForFolders,
					));
				}
			} else {
				return crate::database::PutResult::Err(Box::new(PutError::GetError(get_error)));
			}
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum PutError {
	GetError(super::GetError),
	DoesNotWorksForFolders,
	ContentNotChanged,
	CanNotReadFile {
		path: std::path::PathBuf,
		error: String,
	},
	CanNotWriteFile {
		path: std::path::PathBuf,
		error: String,
	},
	CanNotSerializeFile {
		path: std::path::PathBuf,
		error: String,
	},
	CanNotDeserializeFile {
		path: std::path::PathBuf,
		error: String,
	},
}
impl std::fmt::Display for PutError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::GetError(get_error) => std::fmt::Display::fmt(get_error, f),
			Self::DoesNotWorksForFolders => {
				f.write_str("this method does not works for folders in payload")
			}
			Self::ContentNotChanged => f.write_str("the content has not changed"),
			Self::CanNotReadFile { path, error } => f.write_fmt(format_args!(
				"can not read file `{}` because : {}",
				path.to_string_lossy(),
				error
			)),
			Self::CanNotWriteFile { path, error } => f.write_fmt(format_args!(
				"can not write file `{}` because : {}",
				path.to_string_lossy(),
				error
			)),
			Self::CanNotSerializeFile { path, error } => f.write_fmt(format_args!(
				"can not serialize file `{}` because : {}",
				path.to_string_lossy(),
				error
			)),
			Self::CanNotDeserializeFile { path, error } => f.write_fmt(format_args!(
				"can not deserialize file `{}` because : {}",
				path.to_string_lossy(),
				error
			)),
		}
	}
}
impl std::error::Error for PutError {}
#[cfg(feature = "server_bin")]
impl crate::database::Error for PutError {
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
			Self::ContentNotChanged => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::NOT_MODIFIED,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::CanNotReadFile { path: _, error: _ } => {
				crate::database::build_http_json_response(
					origin,
					&actix_web::http::Method::PUT,
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
					None,
					should_have_body,
				)
			}
			Self::CanNotWriteFile { path: _, error: _ } => {
				crate::database::build_http_json_response(
					origin,
					&actix_web::http::Method::PUT,
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
					None,
					should_have_body,
				)
			}
			Self::CanNotSerializeFile { path: _, error: _ } => {
				crate::database::build_http_json_response(
					origin,
					&actix_web::http::Method::PUT,
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
					None,
					should_have_body,
				)
			}
			Self::CanNotDeserializeFile { path: _, error: _ } => {
				crate::database::build_http_json_response(
					origin,
					&actix_web::http::Method::PUT,
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
					None,
					should_have_body,
				)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(non_snake_case)]

	use std::convert::TryFrom;

	use super::{super::GetError, put, PutError};

	// TODO : test last_modified

	fn build_test_db() -> (tempfile::TempDir, crate::Etag, crate::Etag, crate::Etag) {
		let AA = crate::Item::new_doc(b"AA", "text/plain");
		let A = crate::Item::new_folder(vec![("AA", AA.clone())]);
		let root = crate::Item::new_folder(vec![("A", A.clone())]);

		////////////////////////////////////////////////////////////////////////////////////////////////

		let tmp_folder = tempfile::tempdir().unwrap();
		println!(
			"folder dedicated to this test : {}",
			tmp_folder.path().to_string_lossy()
		);

		let tmp_folder_path = tmp_folder.path().to_path_buf();

		let A_path = tmp_folder_path.join("A");
		let AA_path = A_path.join("AA");

		std::fs::create_dir_all(&A_path).unwrap();

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

		let AA_data_path = A_path.join(".AA.itemdata.toml");
		std::fs::write(
			AA_data_path,
			toml::to_string(&crate::DataDocument::try_from(AA.clone()).unwrap()).unwrap(),
		)
		.unwrap();

		if let crate::Item::Document {
			content: Some(content),
			..
		} = &AA
		{
			std::fs::write(AA_path, content).unwrap();
		} else {
			panic!()
		}

		return (
			tmp_folder,
			root.get_etag().clone(),
			A.get_etag().clone(),
			AA.get_etag().clone(),
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
	fn simple_put_on_not_existing() {
		let tmp_folder = tempfile::tempdir().unwrap();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		let AA_etag = put(
			&tmp_folder_path,
			std::path::Path::new("AA"),
			&crate::Etag::from(""),
			&[],
			crate::Item::new_doc(b"AA", "text/plain"),
		)
		.unwrap();

		let target_content = std::fs::read(&tmp_folder_path.join("AA")).unwrap();
		let target_datafile: crate::DataDocument =
			toml::from_slice(&std::fs::read(&tmp_folder_path.join(".AA.itemdata.toml")).unwrap())
				.unwrap();

		assert_eq!(target_datafile.etag, AA_etag);
		assert_eq!(target_content, b"AA");
		assert_eq!(
			target_datafile.content_type,
			crate::ContentType::from("text/plain")
		);

		tmp_folder.close().unwrap();
	}

	#[test]
	fn simple_put_on_existing() {
		let (tmp_folder, root_etag, A_etag, old_AA_etag) = build_test_db();

		let tmp_folder_path = tmp_folder.path().to_path_buf();

		let AA_etag = put(
			&tmp_folder_path,
			std::path::Path::new("A/AA"),
			&crate::Etag::from(""),
			&[],
			crate::Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap();

		assert_ne!(old_AA_etag, AA_etag);

		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_ne!(root_datafile.etag, root_etag);

		let A_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_ne!(A_datafile.etag, A_etag);

		let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
		let target_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(target_datafile.etag, AA_etag);
		assert_eq!(target_content, b"AA2");
		assert_eq!(
			target_datafile.content_type,
			crate::ContentType::from("text/plain2")
		);

		tmp_folder.close().unwrap();
	}

	#[test]
	fn content_not_changed() {
		let (tmp_folder, root_etag, A_etag, AA_etag) = build_test_db();

		let tmp_folder_path = tmp_folder.path().to_path_buf();

		assert_eq!(
			*put(
				&tmp_folder_path,
				std::path::Path::new("A/AA"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"AA", "text/plain")
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::ContentNotChanged
		);

		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(root_datafile.etag, root_etag);

		let A_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(A_datafile.etag, A_etag);

		let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
		let target_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(target_datafile.etag, AA_etag);
		assert_eq!(target_content, b"AA");
		assert_eq!(
			target_datafile.content_type,
			crate::ContentType::from("text/plain")
		);

		tmp_folder.close().unwrap();
	}

	#[test]
	fn does_not_works_for_folders() {
		let tmp_folder = tempfile::tempdir().unwrap();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		assert_eq!(
			*put(
				&tmp_folder_path,
				std::path::Path::new(""),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_folder(vec![])
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::DoesNotWorksForFolders
		);

		assert_eq!(std::fs::read_dir(&tmp_folder_path).unwrap().count(), 0);

		tmp_folder.close().unwrap();
	}

	#[test]
	fn put_with_if_none_match_all_on_not_existing() {
		let tmp_folder = tempfile::tempdir().unwrap();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		let AA_etag = put(
			&tmp_folder_path,
			std::path::Path::new("A/AA"),
			&crate::Etag::from(""),
			&[&crate::Etag::from("*")],
			crate::Item::new_doc(b"AA", "text/plain"),
		)
		.unwrap();

		let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
		let target_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(target_datafile.etag, AA_etag);
		assert_eq!(target_content, b"AA");
		assert_eq!(
			target_datafile.content_type,
			crate::ContentType::from("text/plain")
		);

		tmp_folder.close().unwrap();
	}

	#[test]
	fn put_with_if_none_match_all_on_existing() {
		let (tmp_folder, root_etag, A_etag, AA_etag) = build_test_db();

		let tmp_folder_path = tmp_folder.path().to_path_buf();

		assert_eq!(
			*put(
				&tmp_folder_path,
				std::path::Path::new("A/AA"),
				&crate::Etag::from(""),
				&[&crate::Etag::from("*")],
				crate::Item::new_doc(b"AA2", "text/plain2"),
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(GetError::IfNoneMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				found: AA_etag.clone(),
				search: crate::Etag::from("*")
			})
		);

		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(root_datafile.etag, root_etag);

		let A_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(A_datafile.etag, A_etag);

		let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
		let target_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(target_datafile.etag, AA_etag);
		assert_eq!(target_content, b"AA");
		assert_eq!(
			target_datafile.content_type,
			crate::ContentType::from("text/plain")
		);

		tmp_folder.close().unwrap();
	}

	#[test]
	fn put_with_if_match_not_found() {
		let (tmp_folder, root_etag, A_etag, AA_etag) = build_test_db();

		let tmp_folder_path = tmp_folder.path().to_path_buf();

		assert_eq!(
			*put(
				&tmp_folder_path,
				std::path::Path::new("A/AA"),
				&crate::Etag::from("ANOTHER_ETAG"),
				&[],
				crate::Item::new_doc(b"AA2", "text/plain2"),
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(GetError::NoIfMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				found: AA_etag.clone(),
				search: crate::Etag::from("ANOTHER_ETAG")
			})
		);

		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(root_datafile.etag, root_etag);

		let A_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(A_datafile.etag, A_etag);

		let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
		let target_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(target_datafile.etag, AA_etag);
		assert_eq!(target_content, b"AA");
		assert_eq!(
			target_datafile.content_type,
			crate::ContentType::from("text/plain")
		);

		tmp_folder.close().unwrap();
	}

	#[test]
	fn put_with_if_match_found() {
		let (tmp_folder, root_etag, A_etag, mut AA_etag) = build_test_db();

		let tmp_folder_path = tmp_folder.path().to_path_buf();

		AA_etag = put(
			&tmp_folder_path,
			std::path::Path::new("A/AA"),
			&AA_etag,
			&[],
			crate::Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap();

		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_ne!(root_datafile.etag, root_etag);

		let A_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_ne!(A_datafile.etag, A_etag);

		let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
		let target_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(target_datafile.etag, AA_etag);
		assert_eq!(target_content, b"AA2");
		assert_eq!(
			target_datafile.content_type,
			crate::ContentType::from("text/plain2")
		);

		tmp_folder.close().unwrap();
	}

	#[test]
	fn put_with_if_match_all() {
		let (tmp_folder, root_etag, A_etag, old_AA_etag) = build_test_db();

		let tmp_folder_path = tmp_folder.path().to_path_buf();

		let AA_etag = put(
			&tmp_folder_path,
			std::path::Path::new("A/AA"),
			&crate::Etag::from("*"),
			&[],
			crate::Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap();

		assert_ne!(old_AA_etag, AA_etag);

		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_ne!(root_datafile.etag, root_etag);

		let A_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_ne!(A_datafile.etag, A_etag);

		let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
		let target_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(target_datafile.etag, AA_etag);
		assert_eq!(target_content, b"AA2");
		assert_eq!(
			target_datafile.content_type,
			crate::ContentType::from("text/plain2")
		);

		tmp_folder.close().unwrap();
	}

	#[test]
	fn put_with_existing_document_conflict() {
		let (tmp_folder, root_etag, A_etag, AA_etag) = build_test_db();

		let tmp_folder_path = tmp_folder.path().to_path_buf();

		assert_eq!(
			*put(
				&tmp_folder_path,
				std::path::Path::new("A/AA/AAA"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"AAA", "text/plain")
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(GetError::Conflict {
				item_path: std::path::PathBuf::from("A/AA")
			})
		);

		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(root_datafile.etag, root_etag);

		let A_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(A_datafile.etag, A_etag);

		let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
		let target_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(target_datafile.etag, AA_etag);
		assert_eq!(target_content, b"AA");
		assert_eq!(
			target_datafile.content_type,
			crate::ContentType::from("text/plain")
		);

		assert!(!tmp_folder_path.join("A/AA/").exists());
		assert!(!tmp_folder_path.join("A/AA/").is_dir());
		assert!(!tmp_folder_path.join("A/AA/.folder.itemdata.toml").exists());
		assert!(!tmp_folder_path.join("A/AA/AAA").exists());
		assert!(!tmp_folder_path.join("A/AA/.AAA.itemdata.toml").exists());

		tmp_folder.close().unwrap();
	}

	#[test]
	fn put_with_existing_folder_conflict() {
		let (tmp_folder, root_etag, A_etag, AA_etag) = build_test_db();

		let tmp_folder_path = tmp_folder.path().to_path_buf();

		assert_eq!(
			*put(
				&tmp_folder_path,
				std::path::Path::new("A"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"A", "text/plain")
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(GetError::Conflict {
				item_path: std::path::PathBuf::from("A")
			})
		);

		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(root_datafile.etag, root_etag);

		let A_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(A_datafile.etag, A_etag);

		let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
		let target_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
		)
		.unwrap();

		assert_eq!(target_datafile.etag, AA_etag);
		assert_eq!(target_content, b"AA");
		assert_eq!(
			target_datafile.content_type,
			crate::ContentType::from("text/plain")
		);

		tmp_folder.close().unwrap();
	}

	#[test]
	fn put_in_public() {
		let tmp_folder = tempfile::tempdir().unwrap();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		let AA_etag = put(
			&tmp_folder_path,
			std::path::Path::new("public/A/AA"),
			&crate::Etag::from(""),
			&[],
			crate::Item::new_doc(b"AA", "text/plain"),
		)
		.unwrap();

		std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap();
		std::fs::read(
			&tmp_folder_path
				.join("public")
				.join("A")
				.join(".folder.itemdata.toml"),
		)
		.unwrap();
		std::fs::read(
			&tmp_folder_path
				.join("public")
				.join("A")
				.join(".folder.itemdata.toml"),
		)
		.unwrap();

		let target_content =
			std::fs::read(&tmp_folder_path.join("public").join("A").join("AA")).unwrap();
		let target_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(
				&tmp_folder_path
					.join("public")
					.join("A")
					.join(".AA.itemdata.toml"),
			)
			.unwrap(),
		)
		.unwrap();

		assert_eq!(target_datafile.etag, AA_etag);
		assert_eq!(target_content, b"AA");
		assert_eq!(
			target_datafile.content_type,
			crate::ContentType::from("text/plain")
		);

		tmp_folder.close().unwrap();
	}

	#[test]
	fn put_in_incorrect_path() {
		let tmp_folder = tempfile::tempdir().unwrap();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		assert_eq!(
			*put(
				&tmp_folder_path,
				std::path::Path::new("A/../AA"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"AA", "text/plain"),
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(GetError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/../"),
				error: String::from("`..` is not allowed")
			})
		);

		assert_eq!(std::fs::read_dir(&tmp_folder_path).unwrap().count(), 0);

		tmp_folder.close().unwrap();
	}
}
