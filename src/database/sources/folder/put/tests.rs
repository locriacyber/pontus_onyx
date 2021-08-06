#![allow(non_snake_case)]

use std::convert::TryFrom;

use super::{super::GetError, put, PutError};
use crate::item::{DataDocument, DataFolder, Etag, Item, ItemPath};

// TODO : test last_modified

fn build_test_db() -> (tempfile::TempDir, Etag, Etag, Etag) {
	let AA = Item::new_doc(b"AA", "text/plain");
	let A = Item::new_folder(vec![("AA", AA.clone())]);
	let root = Item::new_folder(vec![("A", A.clone())]);

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
		toml::to_string(&DataFolder {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: root.get_etag().clone(),
		})
		.unwrap(),
	)
	.unwrap();

	let A_data_path = A_path.join(".folder.itemdata.toml");
	std::fs::write(
		A_data_path,
		toml::to_string(&DataFolder {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: A.get_etag().clone(),
		})
		.unwrap(),
	)
	.unwrap();

	let AA_data_path = A_path.join(".AA.itemdata.toml");
	std::fs::write(
		AA_data_path,
		toml::to_string(&DataDocument::try_from(AA.clone()).unwrap()).unwrap(),
	)
	.unwrap();

	if let Item::Document {
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
		&ItemPath::from("AA"),
		&Etag::from(""),
		&[],
		Item::new_doc(b"AA", "text/plain"),
	)
	.unwrap();

	let target_content = std::fs::read(&tmp_folder_path.join("AA")).unwrap();
	let target_datafile: DataDocument =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".AA.itemdata.toml")).unwrap())
			.unwrap();

	assert_eq!(target_datafile.etag, AA_etag);
	assert_eq!(target_content, b"AA");
	assert_eq!(
		target_datafile.content_type,
		crate::item::ContentType::from("text/plain")
	);

	tmp_folder.close().unwrap();
}

#[test]
fn simple_put_on_existing() {
	let (tmp_folder, root_etag, A_etag, old_AA_etag) = build_test_db();

	let tmp_folder_path = tmp_folder.path().to_path_buf();

	let AA_etag = put(
		&tmp_folder_path,
		&ItemPath::from("A/AA"),
		&Etag::from(""),
		&[],
		Item::new_doc(b"AA2", "text/plain2"),
	)
	.unwrap();

	assert_ne!(old_AA_etag, AA_etag);

	let root_datafile: DataFolder =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap())
			.unwrap();

	assert_ne!(root_datafile.etag, root_etag);

	let A_datafile: DataFolder = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_ne!(A_datafile.etag, A_etag);

	let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
	let target_datafile: DataDocument = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_eq!(target_datafile.etag, AA_etag);
	assert_eq!(target_content, b"AA2");
	assert_eq!(
		target_datafile.content_type,
		crate::item::ContentType::from("text/plain2")
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
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&[],
			Item::new_doc(b"AA", "text/plain")
		)
		.unwrap_err()
		.downcast::<PutError>()
		.unwrap(),
		PutError::ContentNotChanged
	);

	let root_datafile: DataFolder =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap())
			.unwrap();

	assert_eq!(root_datafile.etag, root_etag);

	let A_datafile: DataFolder = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_eq!(A_datafile.etag, A_etag);

	let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
	let target_datafile: DataDocument = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_eq!(target_datafile.etag, AA_etag);
	assert_eq!(target_content, b"AA");
	assert_eq!(
		target_datafile.content_type,
		crate::item::ContentType::from("text/plain")
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
			&ItemPath::from(""),
			&Etag::from(""),
			&[],
			Item::new_folder(vec![])
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
		&ItemPath::from("A/AA"),
		&Etag::from(""),
		&[&Etag::from("*")],
		Item::new_doc(b"AA", "text/plain"),
	)
	.unwrap();

	let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
	let target_datafile: DataDocument = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_eq!(target_datafile.etag, AA_etag);
	assert_eq!(target_content, b"AA");
	assert_eq!(
		target_datafile.content_type,
		crate::item::ContentType::from("text/plain")
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
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&[&Etag::from("*")],
			Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap_err()
		.downcast::<PutError>()
		.unwrap(),
		PutError::GetError(GetError::IfNoneMatch {
			item_path: ItemPath::from("A/AA"),
			found: AA_etag.clone(),
			search: Etag::from("*")
		})
	);

	let root_datafile: DataFolder =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap())
			.unwrap();

	assert_eq!(root_datafile.etag, root_etag);

	let A_datafile: DataFolder = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_eq!(A_datafile.etag, A_etag);

	let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
	let target_datafile: DataDocument = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_eq!(target_datafile.etag, AA_etag);
	assert_eq!(target_content, b"AA");
	assert_eq!(
		target_datafile.content_type,
		crate::item::ContentType::from("text/plain")
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
			&ItemPath::from("A/AA"),
			&Etag::from("ANOTHER_ETAG"),
			&[],
			Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap_err()
		.downcast::<PutError>()
		.unwrap(),
		PutError::GetError(GetError::NoIfMatch {
			item_path: ItemPath::from("A/AA"),
			found: AA_etag.clone(),
			search: Etag::from("ANOTHER_ETAG")
		})
	);

	let root_datafile: DataFolder =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap())
			.unwrap();

	assert_eq!(root_datafile.etag, root_etag);

	let A_datafile: DataFolder = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_eq!(A_datafile.etag, A_etag);

	let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
	let target_datafile: DataDocument = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_eq!(target_datafile.etag, AA_etag);
	assert_eq!(target_content, b"AA");
	assert_eq!(
		target_datafile.content_type,
		crate::item::ContentType::from("text/plain")
	);

	tmp_folder.close().unwrap();
}

#[test]
fn put_with_if_match_found() {
	let (tmp_folder, root_etag, A_etag, mut AA_etag) = build_test_db();

	let tmp_folder_path = tmp_folder.path().to_path_buf();

	AA_etag = put(
		&tmp_folder_path,
		&ItemPath::from("A/AA"),
		&AA_etag,
		&[],
		Item::new_doc(b"AA2", "text/plain2"),
	)
	.unwrap();

	let root_datafile: DataFolder =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap())
			.unwrap();

	assert_ne!(root_datafile.etag, root_etag);

	let A_datafile: DataFolder = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_ne!(A_datafile.etag, A_etag);

	let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
	let target_datafile: DataDocument = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_eq!(target_datafile.etag, AA_etag);
	assert_eq!(target_content, b"AA2");
	assert_eq!(
		target_datafile.content_type,
		crate::item::ContentType::from("text/plain2")
	);

	tmp_folder.close().unwrap();
}

#[test]
fn put_with_if_match_all() {
	let (tmp_folder, root_etag, A_etag, old_AA_etag) = build_test_db();

	let tmp_folder_path = tmp_folder.path().to_path_buf();

	let AA_etag = put(
		&tmp_folder_path,
		&ItemPath::from("A/AA"),
		&Etag::from("*"),
		&[],
		Item::new_doc(b"AA2", "text/plain2"),
	)
	.unwrap();

	assert_ne!(old_AA_etag, AA_etag);

	let root_datafile: DataFolder =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap())
			.unwrap();

	assert_ne!(root_datafile.etag, root_etag);

	let A_datafile: DataFolder = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_ne!(A_datafile.etag, A_etag);

	let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
	let target_datafile: DataDocument = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_eq!(target_datafile.etag, AA_etag);
	assert_eq!(target_content, b"AA2");
	assert_eq!(
		target_datafile.content_type,
		crate::item::ContentType::from("text/plain2")
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
			&ItemPath::from("A/AA/AAA"),
			&Etag::from(""),
			&[],
			Item::new_doc(b"AAA", "text/plain")
		)
		.unwrap_err()
		.downcast::<PutError>()
		.unwrap(),
		PutError::GetError(GetError::Conflict {
			item_path: ItemPath::from("A/AA")
		})
	);

	let root_datafile: DataFolder =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap())
			.unwrap();

	assert_eq!(root_datafile.etag, root_etag);

	let A_datafile: DataFolder = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_eq!(A_datafile.etag, A_etag);

	let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
	let target_datafile: DataDocument = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_eq!(target_datafile.etag, AA_etag);
	assert_eq!(target_content, b"AA");
	assert_eq!(
		target_datafile.content_type,
		crate::item::ContentType::from("text/plain")
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
			&ItemPath::from("A"),
			&Etag::from(""),
			&[],
			Item::new_doc(b"A", "text/plain")
		)
		.unwrap_err()
		.downcast::<PutError>()
		.unwrap(),
		PutError::GetError(GetError::Conflict {
			item_path: ItemPath::from("A/")
		})
	);

	let root_datafile: DataFolder =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap())
			.unwrap();

	assert_eq!(root_datafile.etag, root_etag);

	let A_datafile: DataFolder = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".folder.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_eq!(A_datafile.etag, A_etag);

	let target_content = std::fs::read(&tmp_folder_path.join("A").join("AA")).unwrap();
	let target_datafile: DataDocument = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".AA.itemdata.toml")).unwrap(),
	)
	.unwrap();

	assert_eq!(target_datafile.etag, AA_etag);
	assert_eq!(target_content, b"AA");
	assert_eq!(
		target_datafile.content_type,
		crate::item::ContentType::from("text/plain")
	);

	tmp_folder.close().unwrap();
}

#[test]
fn put_in_public() {
	let tmp_folder = tempfile::tempdir().unwrap();
	let tmp_folder_path = tmp_folder.path().to_path_buf();

	let AA_etag = put(
		&tmp_folder_path,
		&ItemPath::from("public/A/AA"),
		&Etag::from(""),
		&[],
		Item::new_doc(b"AA", "text/plain"),
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
	let target_datafile: DataDocument = toml::from_slice(
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
		crate::item::ContentType::from("text/plain")
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
			&ItemPath::from("A/A\0A"),
			&Etag::from(""),
			&[],
			Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap_err()
		.downcast::<PutError>()
		.unwrap(),
		PutError::GetError(GetError::IncorrectItemName {
			item_path: ItemPath::from("A/A\0A"),
			error: String::from("`A\0A` should not contains `\\0` character")
		})
	);

	assert_eq!(std::fs::read_dir(&tmp_folder_path).unwrap().count(), 0);

	tmp_folder.close().unwrap();
}
