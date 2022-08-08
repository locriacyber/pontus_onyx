#![allow(non_snake_case)]

use super::{super::GetError, super::LocalStorageMock, super::Storage, delete, DeleteError};
use crate::item::{DataDocument, DataFolder, Etag, Item, ItemPath};

fn build_test_db() -> (
	LocalStorageMock,
	String,
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

	let prefix = String::from("pontus_onyx_delete_test");

	let storage = LocalStorageMock::new();

	storage.set_item(&format!("{}/", prefix), "").unwrap();

	storage
		.set_item(
			&format!("{}/.folder.itemdata.json", prefix),
			&serde_json::to_string(&DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: root.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

	/*
	storage.set_item(&format!("{}/A/", prefix), "").unwrap();
	*/

	storage
		.set_item(
			&format!("{}/A/.folder.itemdata.json", prefix),
			&serde_json::to_string(&DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: A.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

	storage
		.set_item(
			&format!("{}/A/AA/.folder.itemdata.json", prefix),
			&serde_json::to_string(&DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: AA.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

	if let Item::Document {
		content: Some(content),
		etag: AAA_etag,
		content_type: AAA_content_type,
		last_modified: AAA_last_modified,
	} = AAA.clone()
	{
		storage
			.set_item(
				&format!("{}/A/AA/.AAA.itemdata.json", prefix),
				&serde_json::to_string(&DataDocument {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: AAA_etag,
					content_type: AAA_content_type,
					last_modified: AAA_last_modified,
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(&format!("{}/A/AA/AAA", prefix), &base64::encode(&content))
			.unwrap();
	} else {
		panic!();
	}

	if let Item::Document {
		content: Some(content),
		etag: AB_etag,
		content_type: AB_content_type,
		last_modified: AB_last_modified,
	} = AB.clone()
	{
		storage
			.set_item(
				&format!("{}/A/.AB.itemdata.json", prefix),
				&serde_json::to_string(&DataDocument {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: AB_etag,
					content_type: AB_content_type,
					last_modified: AB_last_modified,
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(&format!("{}/A/AB", prefix), &base64::encode(&content))
			.unwrap();
	} else {
		panic!();
	}

	/*
	storage
		.set_item(&format!("{}/public/", prefix), "")
		.unwrap();
	*/

	storage
		.set_item(
			&format!("{}/public/.folder.itemdata.json", prefix),
			&serde_json::to_string(&DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: public.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

	/*
	storage.set_item(&format!("{}/public/B/", prefix), "").unwrap();
	*/

	storage
		.set_item(
			&format!("{}/public/B/.folder.itemdata.json", prefix),
			&serde_json::to_string(&DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: B.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

	if let Item::Document {
		content: Some(content),
		etag: BA_etag,
		content_type: BA_content_type,
		last_modified: BA_last_modified,
	} = BA.clone()
	{
		storage
			.set_item(
				&format!("{}/public/B/.BA.itemdata.json", prefix),
				&serde_json::to_string(&DataDocument {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: BA_etag,
					content_type: BA_content_type,
					last_modified: BA_last_modified,
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(
				&format!("{}/public/B/BA", prefix),
				&base64::encode(&content),
			)
			.unwrap();
	} else {
		panic!();
	}

	for i in 0..storage.length().unwrap() {
		dbg!(&storage.key(i).unwrap().unwrap());
	}

	return (
		storage,
		prefix,
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

#[test]
fn simple_delete_on_not_existing() {
	let storage = LocalStorageMock::new();
	let prefix = String::from("pontus_onyx_delete_test");
	storage
		.set_item(
			&format!("{}/.folder.itemdata.json", &prefix),
			&serde_json::to_string(&DataFolder::default()).unwrap(),
		)
		.unwrap();

	for i in 0..storage.length().unwrap() {
		dbg!(&storage.key(i).unwrap().unwrap());
	}

	assert_eq!(
		*delete(
			&storage,
			&prefix,
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

	assert_eq!(storage.length().unwrap(), 1);
	assert_eq!(
		storage.key(0).unwrap().unwrap(),
		format!("{}/.folder.itemdata.json", &prefix)
	);
}

#[test]
fn simple_delete_on_existing() {
	let (storage, prefix, root_etag, A_etag, _, _, AAA_etag, _, _, _) = build_test_db();

	let old_AAA_etag = delete(
		&storage,
		&prefix,
		&ItemPath::from("A/AA/AAA"),
		&Etag::from(""),
	)
	.unwrap();

	assert_eq!(AAA_etag, old_AAA_etag);

	assert_ne!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		root_etag
	);

	assert!(storage
		.get_item(&format!("{}/.A.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert_ne!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		A_etag
	);

	assert!(storage
		.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
		.unwrap()
		.is_none());

	assert!(storage
		.get_item(&format!("{}/A/AA/AAA/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/A/AA/AAA/", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/A/AA/AAA", prefix))
		.unwrap()
		.is_none());

	assert!(storage
		.get_item(&format!("{}/A/.AB.itemdata.json", prefix))
		.unwrap()
		.is_some());
	assert!(storage
		.get_item(&format!("{}/A/AB/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
}

#[test]
fn does_not_works_for_folders() {
	let (storage, prefix, root_etag, A_etag, AA_etag, _, AAA_etag, _, _, _) = build_test_db();

	assert_eq!(
		*delete(&storage, &prefix, &ItemPath::from("A/AA/"), &Etag::from(""),)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
		DeleteError::DoesNotWorksForFolders,
	);

	assert_eq!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		root_etag
	);

	assert!(storage
		.get_item(&format!("{}/.A.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert_eq!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		A_etag
	);

	assert!(storage
		.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
		.unwrap()
		.is_some());
	assert_eq!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		AA_etag
	);

	assert!(storage
		.get_item(&format!("{}/A/AA/AAA/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
		.unwrap()
		.is_some());
	assert_eq!(
		serde_json::from_str::<DataDocument>(
			&storage
				.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		AAA_etag
	);
	assert_eq!(
		base64::decode(
			storage
				.get_item(&format!("{}/A/AA/AAA", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap(),
		b"AAA"
	);

	assert!(storage
		.get_item(&format!("{}/A/.AB.itemdata.json", prefix))
		.unwrap()
		.is_some());
	assert!(storage
		.get_item(&format!("{}/A/AB/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
}

#[test]
fn delete_with_if_match_not_found() {
	let (storage, prefix, root_etag, A_etag, AA_etag, _, AAA_etag, _, _, _) = build_test_db();

	assert_eq!(
		*delete(
			&storage,
			&prefix,
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

	assert_eq!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		root_etag
	);

	assert!(storage
		.get_item(&format!("{}/.A.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert_eq!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		A_etag
	);

	assert!(storage
		.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
		.unwrap()
		.is_some());
	assert_eq!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		AA_etag
	);

	assert!(storage
		.get_item(&format!("{}/A/AA/AAA/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
		.unwrap()
		.is_some());
	assert_eq!(
		serde_json::from_str::<DataDocument>(
			&storage
				.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		AAA_etag
	);
	assert_eq!(
		base64::decode(
			storage
				.get_item(&format!("{}/A/AA/AAA", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap(),
		b"AAA"
	);

	assert!(storage
		.get_item(&format!("{}/A/.AB.itemdata.json", prefix))
		.unwrap()
		.is_some());
	assert!(storage
		.get_item(&format!("{}/A/AB/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
}

#[test]
fn delete_with_if_match_found() {
	let (storage, prefix, root_etag, A_etag, _, _, AAA_etag, _, _, _) = build_test_db();

	let old_AAA_etag = delete(&storage, &prefix, &ItemPath::from("A/AA/AAA"), &AAA_etag).unwrap();

	assert_eq!(old_AAA_etag, AAA_etag);

	assert_ne!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		root_etag
	);

	assert!(storage
		.get_item(&format!("{}/.A.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert_ne!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		A_etag
	);

	assert!(storage
		.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
		.unwrap()
		.is_none());

	assert!(storage
		.get_item(&format!("{}/A/AA/AAA/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/A/AA/AAA/", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/A/AA/AAA", prefix))
		.unwrap()
		.is_none());

	assert!(storage
		.get_item(&format!("{}/A/.AB.itemdata.json", prefix))
		.unwrap()
		.is_some());
	assert!(storage
		.get_item(&format!("{}/A/AB/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
}

#[test]
fn delete_with_if_match_all() {
	let (storage, prefix, root_etag, A_etag, _, _, AAA_etag, _, _, _) = build_test_db();

	let old_AAA_etag = delete(
		&storage,
		&prefix,
		&ItemPath::from("A/AA/AAA"),
		&Etag::from("*"),
	)
	.unwrap();

	assert_eq!(old_AAA_etag, AAA_etag);

	assert_ne!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		root_etag
	);

	assert!(storage
		.get_item(&format!("{}/.A.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert_ne!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		A_etag
	);

	assert!(storage
		.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
		.unwrap()
		.is_none());

	assert!(storage
		.get_item(&format!("{}/A/AA/AAA/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/A/AA/AAA/", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/A/AA/AAA", prefix))
		.unwrap()
		.is_none());

	assert!(storage
		.get_item(&format!("{}/A/.AB.itemdata.json", prefix))
		.unwrap()
		.is_some());
	assert!(storage
		.get_item(&format!("{}/A/AB/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
}

#[test]
fn delete_with_existing_folder_conflict() {
	let (storage, prefix, root_etag, A_etag, AA_etag, _, AAA_etag, _, _, _) = build_test_db();

	assert_eq!(
		*delete(&storage, &prefix, &ItemPath::from("A/AA"), &Etag::from(""),)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
		DeleteError::GetError(GetError::Conflict {
			item_path: ItemPath::from("A/AA/")
		})
	);

	assert_eq!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		root_etag
	);

	assert!(storage
		.get_item(&format!("{}/.A.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert_eq!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		A_etag
	);

	assert!(storage
		.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
		.unwrap()
		.is_some());
	assert_eq!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		AA_etag
	);

	assert!(storage
		.get_item(&format!("{}/A/AA/AAA/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
		.unwrap()
		.is_some());
	assert_eq!(
		serde_json::from_str::<DataDocument>(
			&storage
				.get_item(&format!("{}/A/AA/.AAA.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		AAA_etag
	);
	assert_eq!(
		base64::decode(
			storage
				.get_item(&format!("{}/A/AA/AAA", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap(),
		b"AAA"
	);

	assert!(storage
		.get_item(&format!("{}/A/.AB.itemdata.json", prefix))
		.unwrap()
		.is_some());
	assert!(storage
		.get_item(&format!("{}/A/AB/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
}

#[test]
fn delete_in_public() {
	let (storage, prefix, root_etag, _, _, _, _, _, _, BA_etag) = build_test_db();

	let old_BA_etag = delete(
		&storage,
		&prefix,
		&ItemPath::from("public/B/BA"),
		&Etag::from(""),
	)
	.unwrap();

	assert_eq!(old_BA_etag, BA_etag);

	assert_ne!(
		serde_json::from_str::<DataFolder>(
			&storage
				.get_item(&format!("{}/.folder.itemdata.json", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap()
		.etag,
		root_etag
	);

	assert!(storage
		.get_item(&format!("{}/.public.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/public/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());

	assert!(storage
		.get_item(&format!("{}/public/.B.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/public/B/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());

	assert!(storage
		.get_item(&format!("{}/public/B/.BA.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(storage
		.get_item(&format!("{}/public/B/BA/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
}

#[test]
fn delete_in_incorrect_path() {
	let storage = LocalStorageMock::new();
	let prefix = String::from("pontus_onyx_delete_test");
	storage
		.set_item(
			&format!("{}/.folder.itemdata.json", &prefix),
			&serde_json::to_string(&DataFolder::default()).unwrap(),
		)
		.unwrap();

	for i in 0..storage.length().unwrap() {
		dbg!(&storage.key(i).unwrap().unwrap());
	}

	assert_eq!(
		*delete(
			&storage,
			&prefix,
			&ItemPath::from("A/A\0A"),
			&Etag::from(""),
		)
		.unwrap_err()
		.downcast::<DeleteError>()
		.unwrap(),
		DeleteError::GetError(GetError::IncorrectItemName {
			item_path: ItemPath::from("A/A\0A"),
			error: String::from("`A\0A` should not contains `\\0` character")
		})
	);

	assert_eq!(storage.length().unwrap(), 1);
	assert_eq!(
		storage.key(0).unwrap().unwrap(),
		format!("{}/.folder.itemdata.json", &prefix)
	);
}
