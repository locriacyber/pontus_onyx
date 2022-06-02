#![allow(non_snake_case)]

use super::{super::LocalStorageMock, super::Storage, put, PutError};
use crate::item::{DataDocument, DataFolder, Etag, Item, ItemPath};

// TODO : test last_modified

fn build_test_db() -> (LocalStorageMock, String, Etag, Etag, Etag) {
	let AA = Item::new_doc(b"AA", "text/plain");
	let A = Item::new_folder(vec![("AA", AA.clone())]);
	let root = Item::new_folder(vec![("A", A.clone())]);

	////////////////////////////////////////////////////////////////////////////////////////////////

	let prefix = "pontus_onyx_put_test";

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

	storage.set_item(&format!("{}/A/", prefix), "").unwrap();

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

	if let Item::Document {
		content: Some(content),
		etag: AA_etag,
		content_type: AA_content_type,
		last_modified: AA_last_modified,
	} = AA.clone()
	{
		storage
			.set_item(
				&format!("{}/A/.AA.itemdata.json", prefix),
				&serde_json::to_string(&DataDocument {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: AA_etag,
					content_type: AA_content_type,
					last_modified: AA_last_modified,
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(&format!("{}/A/AA", prefix), &base64::encode(&content))
			.unwrap();
	} else {
		panic!()
	}

	return (
		storage,
		String::from(prefix),
		root.get_etag().clone(),
		A.get_etag().clone(),
		AA.get_etag().clone(),
	);
}

#[test]
fn simple_put_on_not_existing() {
	let storage = LocalStorageMock::new();
	let prefix = String::from("pontus_onyx_put_test");

	let (AA_etag, _) = put(
		&storage,
		&prefix,
		&ItemPath::from("AA"),
		&Etag::from(""),
		&[],
		Item::new_doc(b"AA", "text/plain"),
	)
	.unwrap();

	assert!(storage
		.get_item(&format!("{}/.folder.itemdata.json", prefix))
		.unwrap()
		.is_some());
	let AA_datadoc: DataDocument = serde_json::from_str(
		&storage
			.get_item(&format!("{}/.AA.itemdata.json", prefix))
			.unwrap()
			.unwrap(),
	)
	.unwrap();
	assert_eq!(AA_datadoc.etag, AA_etag);
	assert_eq!(AA_datadoc.content_type, "text/plain");
	assert_eq!(
		base64::decode(
			storage
				.get_item(&format!("{}/AA", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap(),
		b"AA"
	);
}

#[test]
fn simple_put_on_existing() {
	let (storage, prefix, root_etag, A_etag, old_AA_etag) = build_test_db();

	let (AA_etag, _) = put(
		&storage,
		&prefix,
		&ItemPath::from("A/AA"),
		&Etag::from(""),
		&[],
		Item::new_doc(b"AA2", "text/plain2"),
	)
	.unwrap();

	assert_ne!(old_AA_etag, AA_etag);

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
	let AA_datadoc: DataDocument = serde_json::from_str(
		&storage
			.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
			.unwrap()
			.unwrap(),
	)
	.unwrap();
	assert_eq!(AA_datadoc.etag, AA_etag);
	assert_eq!(AA_datadoc.content_type, "text/plain2");
	assert_eq!(
		base64::decode(
			storage
				.get_item(&format!("{}/A/AA", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap(),
		b"AA2"
	);
}

#[test]
fn content_not_changed() {
	let (storage, prefix, root_etag, A_etag, AA_etag) = build_test_db();

	assert_eq!(
		*put(
			&storage,
			&prefix,
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
		.is_none());
	let AA_datadoc: DataDocument = serde_json::from_str(
		&storage
			.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
			.unwrap()
			.unwrap(),
	)
	.unwrap();
	assert_eq!(AA_datadoc.etag, AA_etag);
	assert_eq!(AA_datadoc.content_type, "text/plain");
	assert_eq!(
		base64::decode(
			storage
				.get_item(&format!("{}/A/AA", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap(),
		b"AA"
	);
}

#[test]
fn does_not_works_for_folders() {
	let prefix = "pontus_onyx_get_test";
	let storage = LocalStorageMock::new();

	assert_eq!(
		*put(
			&storage,
			&prefix,
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

	assert_eq!(storage.length().unwrap(), 0);
}

#[test]
fn put_with_if_none_match_all_on_not_existing() {
	let storage = LocalStorageMock::new();
	let prefix = String::from("pontus_onyx_put_test");

	let (AA_etag, _) = put(
		&storage,
		&prefix,
		&ItemPath::from("A/AA"),
		&Etag::from(""),
		&[&Etag::from("*")],
		Item::new_doc(b"AA", "text/plain"),
	)
	.unwrap();

	assert!(serde_json::from_str::<DataFolder>(
		&storage
			.get_item(&format!("{}/.folder.itemdata.json", prefix))
			.unwrap()
			.unwrap()
	)
	.is_ok());

	assert!(storage
		.get_item(&format!("{}/.A.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(serde_json::from_str::<DataFolder>(
		&storage
			.get_item(&format!("{}/A/.folder.itemdata.json", prefix))
			.unwrap()
			.unwrap()
	)
	.is_ok());

	assert!(storage
		.get_item(&format!("{}/A/AA/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
	let AA_datadoc: DataDocument = serde_json::from_str(
		&storage
			.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
			.unwrap()
			.unwrap(),
	)
	.unwrap();
	assert_eq!(AA_datadoc.etag, AA_etag);
	assert_eq!(AA_datadoc.content_type, "text/plain");
	assert_eq!(
		base64::decode(
			storage
				.get_item(&format!("{}/A/AA", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap(),
		b"AA"
	);
}

#[test]
fn put_with_if_none_match_all_on_existing() {
	let (storage, prefix, root_etag, A_etag, AA_etag) = build_test_db();

	assert_eq!(
		*put(
			&storage,
			&prefix,
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&[&Etag::from("*")],
			Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap_err()
		.downcast::<PutError>()
		.unwrap(),
		PutError::GetError(super::super::GetError::IfNoneMatch {
			item_path: ItemPath::from("A/AA"),
			found: AA_etag.clone(),
			search: Etag::from("*")
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
		.is_none());
	let AA_datadoc: DataDocument = serde_json::from_str(
		&storage
			.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
			.unwrap()
			.unwrap(),
	)
	.unwrap();
	assert_eq!(AA_datadoc.etag, AA_etag);
	assert_eq!(AA_datadoc.content_type, "text/plain");
	assert_eq!(
		base64::decode(
			storage
				.get_item(&format!("{}/A/AA", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap(),
		b"AA"
	);
}

#[test]
fn put_with_if_match_not_found() {
	let (storage, prefix, root_etag, A_etag, AA_etag) = build_test_db();

	assert_eq!(
		*put(
			&storage,
			&prefix,
			&ItemPath::from("A/AA"),
			&Etag::from("ANOTHER_ETAG"),
			&[],
			Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap_err()
		.downcast::<PutError>()
		.unwrap(),
		PutError::GetError(super::super::GetError::NoIfMatch {
			item_path: ItemPath::from("A/AA"),
			found: AA_etag.clone(),
			search: Etag::from("ANOTHER_ETAG")
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
		.is_none());
	let AA_datadoc: DataDocument = serde_json::from_str(
		&storage
			.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
			.unwrap()
			.unwrap(),
	)
	.unwrap();
	assert_eq!(AA_datadoc.etag, AA_etag);
	assert_eq!(AA_datadoc.content_type, "text/plain");
	assert_eq!(
		base64::decode(
			storage
				.get_item(&format!("{}/A/AA", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap(),
		b"AA"
	);
}

#[test]
fn put_with_if_match_found() {
	let (storage, prefix, root_etag, A_etag, mut AA_etag) = build_test_db();

	(AA_etag, _) = put(
		&storage,
		&prefix,
		&ItemPath::from("A/AA"),
		&AA_etag,
		&[],
		Item::new_doc(b"AA2", "text/plain2"),
	)
	.unwrap();

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
	let AA_datadoc: DataDocument = serde_json::from_str(
		&storage
			.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
			.unwrap()
			.unwrap(),
	)
	.unwrap();
	assert_eq!(AA_datadoc.etag, AA_etag);
	assert_eq!(AA_datadoc.content_type, "text/plain2");
	assert_eq!(
		base64::decode(
			storage
				.get_item(&format!("{}/A/AA", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap(),
		b"AA2"
	);
}

#[test]
fn put_with_if_match_all() {
	let (storage, prefix, root_etag, A_etag, old_AA_etag) = build_test_db();

	let (AA_etag, _) = put(
		&storage,
		&prefix,
		&ItemPath::from("A/AA"),
		&Etag::from("*"),
		&[],
		Item::new_doc(b"AA2", "text/plain2"),
	)
	.unwrap();

	assert_ne!(old_AA_etag, AA_etag);

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
	let AA_datadoc: DataDocument = serde_json::from_str(
		&storage
			.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
			.unwrap()
			.unwrap(),
	)
	.unwrap();
	assert_eq!(AA_datadoc.etag, AA_etag);
	assert_eq!(AA_datadoc.content_type, "text/plain2");
	assert_eq!(
		base64::decode(
			storage
				.get_item(&format!("{}/A/AA", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap(),
		b"AA2"
	);
}

#[test]
fn put_with_existing_document_conflict() {
	let (storage, prefix, root_etag, A_etag, AA_etag) = build_test_db();

	assert_eq!(
		*put(
			&storage,
			&prefix,
			&ItemPath::from("A/AA/AAA"),
			&Etag::from(""),
			&[],
			Item::new_doc(b"AAA", "text/plain")
		)
		.unwrap_err()
		.downcast::<PutError>()
		.unwrap(),
		PutError::GetError(super::super::GetError::Conflict {
			item_path: ItemPath::from("A/AA")
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
		.is_none());
	let AA_datadoc: DataDocument = serde_json::from_str(
		&storage
			.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
			.unwrap()
			.unwrap(),
	)
	.unwrap();
	assert_eq!(AA_datadoc.etag, AA_etag);
	assert_eq!(AA_datadoc.content_type, "text/plain");
	assert_eq!(
		base64::decode(
			storage
				.get_item(&format!("{}/A/AA", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap(),
		b"AA"
	);
}

#[test]
fn put_with_existing_folder_conflict() {
	let (storage, prefix, root_etag, A_etag, AA_etag) = build_test_db();

	assert_eq!(
		*put(
			&storage,
			&prefix,
			&ItemPath::from("A"),
			&Etag::from(""),
			&[],
			Item::new_doc(b"A", "text/plain")
		)
		.unwrap_err()
		.downcast::<PutError>()
		.unwrap(),
		PutError::GetError(super::super::GetError::Conflict {
			item_path: ItemPath::from("A/")
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
		.is_none());
	let AA_datadoc: DataDocument = serde_json::from_str(
		&storage
			.get_item(&format!("{}/A/.AA.itemdata.json", prefix))
			.unwrap()
			.unwrap(),
	)
	.unwrap();
	assert_eq!(AA_datadoc.etag, AA_etag);
	assert_eq!(AA_datadoc.content_type, "text/plain");
	assert_eq!(
		base64::decode(
			storage
				.get_item(&format!("{}/A/AA", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap(),
		b"AA"
	);
}

#[test]
fn put_in_public() {
	let storage = LocalStorageMock::new();
	let prefix = String::from("pontus_onyx_put_test");

	let (AA_etag, _) = put(
		&storage,
		&prefix,
		&ItemPath::from("public/A/AA"),
		&Etag::from(""),
		&[],
		Item::new_doc(b"AA", "text/plain"),
	)
	.unwrap();

	assert!(serde_json::from_str::<DataFolder>(
		&storage
			.get_item(&format!("{}/.folder.itemdata.json", prefix))
			.unwrap()
			.unwrap()
	)
	.is_ok());

	assert!(storage
		.get_item(&format!("{}/.public.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(serde_json::from_str::<DataFolder>(
		&storage
			.get_item(&format!("{}/public/.folder.itemdata.json", prefix))
			.unwrap()
			.unwrap()
	)
	.is_ok());

	assert!(storage
		.get_item(&format!("{}/public/.A.itemdata.json", prefix))
		.unwrap()
		.is_none());
	assert!(serde_json::from_str::<DataFolder>(
		&storage
			.get_item(&format!("{}/public/A/.folder.itemdata.json", prefix))
			.unwrap()
			.unwrap()
	)
	.is_ok());

	assert!(storage
		.get_item(&format!("{}/public/A/AA/.folder.itemdata.json", prefix))
		.unwrap()
		.is_none());
	let AA_datadoc: DataDocument = serde_json::from_str(
		&storage
			.get_item(&format!("{}/public/A/.AA.itemdata.json", prefix))
			.unwrap()
			.unwrap(),
	)
	.unwrap();
	assert_eq!(AA_datadoc.etag, AA_etag);
	assert_eq!(AA_datadoc.content_type, "text/plain");
	assert_eq!(
		base64::decode(
			storage
				.get_item(&format!("{}/public/A/AA", prefix))
				.unwrap()
				.unwrap()
		)
		.unwrap(),
		b"AA"
	);
}

#[test]
fn put_in_incorrect_path() {
	let storage = LocalStorageMock::new();
	let prefix = String::from("pontus_onyx_put_test");

	assert_eq!(
		*put(
			&storage,
			&prefix,
			&ItemPath::from("A/A\0A"),
			&Etag::from(""),
			&[],
			Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap_err()
		.downcast::<PutError>()
		.unwrap(),
		PutError::GetError(super::super::GetError::IncorrectItemName {
			item_path: ItemPath::from("A/A\0A"),
			error: String::from("`A\0A` should not contains `\\0` character")
		})
	);

	assert_eq!(storage.length().unwrap(), 0);
}
