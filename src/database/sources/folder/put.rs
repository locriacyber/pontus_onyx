pub fn put(
	_root_folder_path: &std::path::Path,
	_path: &std::path::Path,
	_if_match: &crate::Etag,
	_if_none_match: &[&crate::Etag],
	_item: crate::Item,
) -> crate::database::PutResult {
	todo!()
}

#[derive(Debug, PartialEq, Eq)]
pub enum PutError {
	Conflict {
		item_path: std::path::PathBuf,
	},
	NoContentInside {
		item_path: std::path::PathBuf,
	},
	IncorrectItemName {
		item_path: std::path::PathBuf,
		error: String,
	},
	NoIfMatch {
		item_path: std::path::PathBuf,
		search: crate::Etag,
		found: crate::Etag,
	},
	IfNoneMatch {
		item_path: std::path::PathBuf,
		search: crate::Etag,
		found: crate::Etag,
	},
	DoesNotWorksForFolders,
	InternalError,
	ContentNotChanged,
	CanNotFetchParent {
		item_path: std::path::PathBuf,
		error: super::GetError,
	},
}
impl std::fmt::Display for PutError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Conflict{item_path} => f.write_fmt(format_args!("name conflict between folder and file on the path `{}`", item_path.to_string_lossy())),
			Self::NoContentInside{item_path} => f.write_fmt(format_args!("no content found in `{}`", item_path.to_string_lossy())),
			Self::IncorrectItemName{item_path, error} => f.write_fmt(format_args!("the path `{}` is incorrect, because {}", item_path.to_string_lossy(), error)),
			Self::NoIfMatch{item_path, search, found} => f.write_fmt(format_args!("the requested `{}` etag (through `IfMatch`) for `{}` was not found, found `{}` instead", search, item_path.to_string_lossy(), found)),
			Self::IfNoneMatch{item_path, search, found} => f.write_fmt(format_args!("the unwanted etag `{}` (through `IfNoneMatch`) for `{}` was matches with `{}`", search, item_path.to_string_lossy(), found)),
			Self::DoesNotWorksForFolders => f.write_str("this method does not works on folders"),
			Self::InternalError => f.write_str("internal server error"),
			Self::ContentNotChanged => f.write_str("content not changed"),
			Self::CanNotFetchParent { item_path, error } => f.write_fmt(format_args!("can not fetch parent of `{}`, because : `{}`", item_path.to_string_lossy(), error)),
		}
	}
}
impl std::error::Error for PutError {}
impl crate::database::Error for PutError {
	fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse {
		match self {
			Self::Conflict { item_path: _ } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::CONFLICT,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::NoContentInside { item_path: _ } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::IncorrectItemName {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::NoIfMatch {
				item_path: _,
				search: _,
				found: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::IfNoneMatch {
				item_path: _,
				search: _,
				found: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::DoesNotWorksForFolders => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::InternalError => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
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
			Self::CanNotFetchParent {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(non_snake_case)]

	use std::convert::TryFrom;

	use super::{put, PutError};

	// TODO : test last_modified

	fn build_test_db() -> (tempfile::TempDir, crate::Etag, crate::Etag, crate::Etag) {
		let AA = crate::Item::new_doc(b"AA", "text/plain");

		let A = crate::Item::new_folder(vec![("AA", AA.clone())]);

		let root = crate::Item::new_folder(vec![("A", A.clone())]);

		let mut root_without_public = root.clone();
		if let crate::Item::Folder {
			content: Some(content),
			..
		} = &mut root_without_public
		{
			if let crate::Item::Folder {
				content: public_content,
				..
			} = &mut **content.get_mut("public").unwrap()
			{
				*public_content = None;
			}
		} else {
			panic!()
		}

		////////////////////////////////////////////////////////////////////////////////////////////////

		let tmp_folder = tempfile::tempdir().unwrap();
		println!("folder dedicated to this test : {:?}", tmp_folder.path());

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

	#[test]
	fn simple_put_on_not_existing() {
		let tmp_folder = tempfile::tempdir().unwrap();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		let AA_etag = put(
			&tmp_folder_path,
			&std::path::PathBuf::from("AA"),
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
			&std::path::PathBuf::from("A/AA"),
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
				&std::path::PathBuf::from("A/AA"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"AA2", "text/plain2")
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
				&std::path::PathBuf::from(""),
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
			&std::path::PathBuf::from("A/AA"),
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
				&std::path::PathBuf::from("A/AA"),
				&crate::Etag::from(""),
				&[&crate::Etag::from("*")],
				crate::Item::new_doc(b"AA2", "text/plain2"),
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::IfNoneMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				found: AA_etag.clone(),
				search: crate::Etag::from("*"),
			}
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
				&std::path::PathBuf::from("A/AA"),
				&crate::Etag::from("ANOTHER_ETAG"),
				&[],
				crate::Item::new_doc(b"AA2", "text/plain2"),
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::NoIfMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				found: AA_etag.clone(),
				search: crate::Etag::from("ANOTHER_ETAG"),
			}
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
			&std::path::PathBuf::from("A/AA"),
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
			&std::path::PathBuf::from("A/AA"),
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
	fn put_with_existing_folder_conflict() {
		let (tmp_folder, root_etag, A_etag, AA_etag) = build_test_db();

		let tmp_folder_path = tmp_folder.path().to_path_buf();

		assert_eq!(
			*put(
				&tmp_folder_path,
				&std::path::PathBuf::from("A"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"A", "text/plain")
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::Conflict {
				item_path: std::path::PathBuf::from("A")
			}
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
			&std::path::PathBuf::from("public/A/AA"),
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
				&std::path::PathBuf::from("A/../AA"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"AA", "text/plain"),
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/../"),
				error: String::from("`..` is not allowed")
			}
		);

		assert_eq!(std::fs::read_dir(&tmp_folder_path).unwrap().count(), 0);

		tmp_folder.close().unwrap();
	}
}
