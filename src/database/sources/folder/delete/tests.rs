#![allow(non_snake_case)]

use std::convert::TryFrom;

use super::{super::GetError, delete, DeleteError};
use crate::item::{DataDocument, DataFolder, Etag, Item, ItemPath};

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

	let AA_data_path = AA_path.join(".folder.itemdata.toml");
	std::fs::write(
		AA_data_path,
		toml::to_string(&DataFolder {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: AA.get_etag().clone(),
		})
		.unwrap(),
	)
	.unwrap();

	let AB_data_path = A_path.join(".AB.itemdata.toml");
	std::fs::write(
		AB_data_path,
		toml::to_string(&DataDocument::try_from(AB.clone()).unwrap()).unwrap(),
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
		toml::to_string(&DataDocument::try_from(AAA.clone()).unwrap()).unwrap(),
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
		toml::to_string(&DataFolder {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: public.get_etag().clone(),
		})
		.unwrap(),
	)
	.unwrap();

	let B_data_path = B_path.join(".folder.itemdata.toml");
	std::fs::write(
		B_data_path,
		toml::to_string(&DataFolder {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: B.get_etag().clone(),
		})
		.unwrap(),
	)
	.unwrap();

	let BA_data_path = B_path.join(".BA.itemdata.toml");
	std::fs::write(
		BA_data_path,
		toml::to_string(&DataDocument::try_from(BA.clone()).unwrap()).unwrap(),
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
	let root_datafile: DataFolder =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap())
			.unwrap();
	assert_ne!(root_datafile.etag, root_etag);

	assert!(tmp_folder_path.join("A").exists());
	assert!(tmp_folder_path
		.join("A")
		.join(".folder.itemdata.toml")
		.exists());
	let A_datafile: DataFolder = toml::from_slice(
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
	let AB_datafile: DataDocument = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
	)
	.unwrap();
	assert_eq!(AB_datafile.etag, AB_etag);

	assert!(tmp_folder_path.join("public").exists());
	assert!(tmp_folder_path
		.join("public")
		.join(".folder.itemdata.toml")
		.exists());
	let public_datafile: DataFolder = toml::from_slice(
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
	let B_datafile: DataFolder = toml::from_slice(
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
	let BA_datafile: DataDocument = toml::from_slice(
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
	let (tmp_folder, root_etag, A_etag, AA_etag, AB_etag, AAA_etag, public_etag, B_etag, BA_etag) =
		build_test_db();
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
	let root_datafile: DataFolder =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap())
			.unwrap();
	assert_eq!(root_datafile.etag, root_etag);

	assert!(tmp_folder_path.join("A").exists());
	assert!(tmp_folder_path
		.join("A")
		.join(".folder.itemdata.toml")
		.exists());
	let A_datafile: DataFolder = toml::from_slice(
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
	let AA_datafile: DataFolder = toml::from_slice(
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
	let AAA_datafile: DataDocument = toml::from_slice(
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
	let AB_datafile: DataDocument = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
	)
	.unwrap();
	assert_eq!(AB_datafile.etag, AB_etag);

	assert!(tmp_folder_path.join("public").exists());
	assert!(tmp_folder_path
		.join("public")
		.join(".folder.itemdata.toml")
		.exists());
	let public_datafile: DataFolder = toml::from_slice(
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
	let B_datafile: DataFolder = toml::from_slice(
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
	let BA_datafile: DataDocument = toml::from_slice(
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
	let (tmp_folder, root_etag, A_etag, AA_etag, AB_etag, AAA_etag, public_etag, B_etag, BA_etag) =
		build_test_db();
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
	let root_datafile: DataFolder =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap())
			.unwrap();
	assert_eq!(root_datafile.etag, root_etag);

	assert!(tmp_folder_path.join("A").exists());
	assert!(tmp_folder_path
		.join("A")
		.join(".folder.itemdata.toml")
		.exists());
	let A_datafile: DataFolder = toml::from_slice(
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
	let AA_datafile: DataFolder = toml::from_slice(
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
	let AAA_datafile: DataDocument = toml::from_slice(
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
	let AB_datafile: DataDocument = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
	)
	.unwrap();
	assert_eq!(AB_datafile.etag, AB_etag);

	assert!(tmp_folder_path.join("public").exists());
	assert!(tmp_folder_path
		.join("public")
		.join(".folder.itemdata.toml")
		.exists());
	let public_datafile: DataFolder = toml::from_slice(
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
	let B_datafile: DataFolder = toml::from_slice(
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
	let BA_datafile: DataDocument = toml::from_slice(
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

	let old_AAA_etag = delete(&tmp_folder_path, &ItemPath::from("A/AA/AAA"), &AAA_etag).unwrap();

	assert_eq!(old_AAA_etag, AAA_etag);

	assert!(tmp_folder_path.exists());
	assert!(tmp_folder_path.join(".folder.itemdata.toml").exists());
	let root_datafile: DataFolder =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap())
			.unwrap();
	assert_ne!(root_datafile.etag, root_etag);

	assert!(tmp_folder_path.join("A").exists());
	assert!(tmp_folder_path
		.join("A")
		.join(".folder.itemdata.toml")
		.exists());
	let A_datafile: DataFolder = toml::from_slice(
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
	let AB_datafile: DataDocument = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
	)
	.unwrap();
	assert_eq!(AB_datafile.etag, AB_etag);

	assert!(tmp_folder_path.join("public").exists());
	assert!(tmp_folder_path
		.join("public")
		.join(".folder.itemdata.toml")
		.exists());
	let public_datafile: DataFolder = toml::from_slice(
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
	let B_datafile: DataFolder = toml::from_slice(
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
	let BA_datafile: DataDocument = toml::from_slice(
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
	let root_datafile: DataFolder =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap())
			.unwrap();
	assert_ne!(root_datafile.etag, root_etag);

	assert!(tmp_folder_path.join("A").exists());
	assert!(tmp_folder_path
		.join("A")
		.join(".folder.itemdata.toml")
		.exists());
	let A_datafile: DataFolder = toml::from_slice(
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
	let AB_datafile: DataDocument = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
	)
	.unwrap();
	assert_eq!(AB_datafile.etag, AB_etag);

	assert!(tmp_folder_path.join("public").exists());
	assert!(tmp_folder_path
		.join("public")
		.join(".folder.itemdata.toml")
		.exists());
	let public_datafile: DataFolder = toml::from_slice(
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
	let B_datafile: DataFolder = toml::from_slice(
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
	let BA_datafile: DataDocument = toml::from_slice(
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
	let (tmp_folder, root_etag, A_etag, AA_etag, AB_etag, AAA_etag, public_etag, B_etag, BA_etag) =
		build_test_db();
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
	let root_datafile: DataFolder =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap())
			.unwrap();
	assert_eq!(root_datafile.etag, root_etag);

	assert!(tmp_folder_path.join("A").exists());
	assert!(tmp_folder_path
		.join("A")
		.join(".folder.itemdata.toml")
		.exists());
	let A_datafile: DataFolder = toml::from_slice(
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
	let AA_datafile: DataFolder = toml::from_slice(
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
	let AAA_datafile: DataDocument = toml::from_slice(
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
	let AB_datafile: DataDocument = toml::from_slice(
		&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
	)
	.unwrap();
	assert_eq!(AB_datafile.etag, AB_etag);

	assert!(tmp_folder_path.join("public").exists());
	assert!(tmp_folder_path
		.join("public")
		.join(".folder.itemdata.toml")
		.exists());
	let public_datafile: DataFolder = toml::from_slice(
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
	let B_datafile: DataFolder = toml::from_slice(
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
	let BA_datafile: DataDocument = toml::from_slice(
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
	let root_datafile: DataFolder =
		toml::from_slice(&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap())
			.unwrap();
	assert_ne!(root_datafile.etag, root_etag);

	assert!(tmp_folder_path.join("A").exists());
	assert!(tmp_folder_path
		.join("A")
		.join(".folder.itemdata.toml")
		.exists());
	let A_datafile: DataFolder = toml::from_slice(
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
	let AA_datafile: DataFolder = toml::from_slice(
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
	let AAA_datafile: DataDocument = toml::from_slice(
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
	let AB_datafile: DataDocument = toml::from_slice(
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
