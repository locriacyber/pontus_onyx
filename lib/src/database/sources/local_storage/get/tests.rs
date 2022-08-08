#![allow(non_snake_case)]

use super::{super::LocalStorageMock, super::Storage, get, GetError};
use crate::item::{DataDocument, DataFolder, Etag, Item, ItemPath};

// TODO : test if folderdata found but content is file
// TODO : test if filedata found but content is folder

#[test]
fn all_tests_bulk() {
	let AA = Item::new_doc(b"AA", "text/plain");
	let AB = Item::new_doc(b"AB", "text/plain");
	let AC = Item::new_doc(b"AC", "text/plain");
	let BA = Item::new_doc(b"BA", "text/plain");
	let BB = Item::new_doc(b"BB", "text/plain");
	let CA = Item::new_doc(b"CA", "text/plain");

	let A = Item::new_folder(vec![
		("AA", AA.clone()),
		("AB", AB.clone()),
		("AC", AC.clone()),
	]);
	let B = Item::new_folder(vec![("BA", BA.clone()), ("BB", BB.clone())]);
	let C = Item::new_folder(vec![("CA", CA.clone())]);
	let public = Item::new_folder(vec![("C", C.clone())]);

	let root = Item::new_folder(vec![
		("A", A.clone()),
		("B", B.clone()),
		("public", public.clone()),
	]);

	let mut root_without_public = root.clone();
	if let Item::Folder {
		content: Some(content),
		..
	} = &mut root_without_public
	{
		content.remove("public").unwrap();
	} else {
		panic!()
	}

	////////////////////////////////////////////////////////////////////////////////////////////////

	let prefix = "pontus_onyx_get_test";

	let storage = LocalStorageMock::new();

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
	}

	if let Item::Document {
		content: Some(content),
		etag: AC_etag,
		content_type: AC_content_type,
		last_modified: AC_last_modified,
	} = AC.clone()
	{
		storage
			.set_item(
				&format!("{}/A/.AC.itemdata.json", prefix),
				&serde_json::to_string(&DataDocument {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: AC_etag,
					content_type: AC_content_type,
					last_modified: AC_last_modified,
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(&format!("{}/A/AC", prefix), &base64::encode(&content))
			.unwrap();
	}

	storage
		.set_item(
			&format!("{}/B/.folder.itemdata.json", prefix),
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
				&format!("{}/B/.BA.itemdata.json", prefix),
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
			.set_item(&format!("{}/B/BA", prefix), &base64::encode(&content))
			.unwrap();
	}

	if let Item::Document {
		content: Some(content),
		etag: BB_etag,
		content_type: BB_content_type,
		last_modified: BB_last_modified,
	} = BB.clone()
	{
		storage
			.set_item(
				&format!("{}/B/.BB.itemdata.json", prefix),
				&serde_json::to_string(&DataDocument {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: BB_etag,
					content_type: BB_content_type,
					last_modified: BB_last_modified,
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(&format!("{}/B/BB", prefix), &base64::encode(&content))
			.unwrap();
	}

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

	storage
		.set_item(
			&format!("{}/public/C/.folder.itemdata.json", prefix),
			&serde_json::to_string(&DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: C.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

	if let Item::Document {
		content: Some(content),
		etag: CA_etag,
		content_type: CA_content_type,
		last_modified: CA_last_modified,
	} = CA.clone()
	{
		storage
			.set_item(
				&format!("{}/public/C/.CA.itemdata.json", prefix),
				&serde_json::to_string(&DataDocument {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: CA_etag,
					content_type: CA_content_type,
					last_modified: CA_last_modified,
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(
				&format!("{}/public/C/CA", prefix),
				&base64::encode(&content),
			)
			.unwrap();
	}

	let mut keys = vec![];
	for id in 0..storage.length().unwrap() {
		let key = storage.key(id).unwrap().unwrap();
		keys.push(key);
	}
	keys.sort();
	dbg!(keys);

	////////////////////////////////////////////////////////////////////////////////////////////////

	println!("//////// 010 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from(""),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap(),
		root_without_public.clone()
	);
	println!("//////// 020 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("A/"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap(),
		A.clone()
	);
	println!("//////// 030 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap(),
		AA.clone()
	);
	println!("//////// 040 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("A/AB"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap(),
		AB
	);
	println!("//////// 050 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("A/AC"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap(),
		AC
	);
	println!("//////// 060 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("B/"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap(),
		B
	);
	println!("//////// 070 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("B/BA"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap(),
		BA
	);
	println!("//////// 080 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("B/BB"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap(),
		BB
	);
	println!("//////// 090 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("public/C/CA"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap(),
		CA
	);

	////////////////////////////////////////////////////////////////////////////////////////////////

	println!("//////// 100 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from(""),
			root.get_etag(),
			&vec![],
			true
		)
		.unwrap(),
		root_without_public.clone()
	);
	println!("//////// 110 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("A/"),
			A.get_etag(),
			&vec![],
			true
		)
		.unwrap(),
		A.clone()
	);
	println!("//////// 120 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("A/AA"),
			AA.get_etag(),
			&vec![],
			true
		)
		.unwrap(),
		AA.clone()
	);

	////////////////////////////////////////////////////////////////////////////////////////////////

	println!("//////// 130 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from(""),
			&Etag::from(""),
			&[&Etag::from("ANOTHER_ETAG")],
			true
		)
		.unwrap(),
		root_without_public.clone()
	);
	println!("//////// 140 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("A/"),
			&Etag::from(""),
			&[&Etag::from("ANOTHER_ETAG")],
			true
		)
		.unwrap(),
		A.clone()
	);
	println!("//////// 150 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&[&Etag::from("ANOTHER_ETAG")],
			true
		)
		.unwrap(),
		AA.clone()
	);

	////////////////////////////////////////////////////////////////////////////////////////////////

	println!("//////// 160 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::Conflict {
			item_path: ItemPath::from("A/")
		}
	);
	println!("//////// 170 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A/AA/"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::Conflict {
			item_path: ItemPath::from("A/AA")
		}
	);
	println!("//////// 180 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A/AC/not_exists"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::Conflict {
			item_path: ItemPath::from("A/AC")
		}
	);
	println!("//////// 190 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A/not_exists"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::NotFound {
			item_path: ItemPath::from("A/not_exists")
		}
	);
	println!("//////// 200 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A/not_exists/nested"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::NotFound {
			item_path: ItemPath::from("A/not_exists/")
		}
	);
	println!("//////// 210 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("B/not_exists"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::NotFound {
			item_path: ItemPath::from("B/not_exists")
		}
	);
	println!("//////// 220 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("not_exists/"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::NotFound {
			item_path: ItemPath::from("not_exists/")
		}
	);
	println!("//////// 230 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("not_exists"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::NotFound {
			item_path: ItemPath::from("not_exists")
		}
	);
	/*
	// useless with `ItemPath`
	println!("//////// 240 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("."),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::IncorrectItemName {
			item_path: ItemPath::from("."),
			error: String::from("`.` is not allowed")
		}
	);
	*/
	println!("//////// 245 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("."),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap(),
		root_without_public,
	);
	/*
	// useless with `ItemPath`
	println!("//////// 250 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A/.."),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::IncorrectItemName {
			item_path: ItemPath::from("A/.."),
			error: String::from("`..` is not allowed")
		}
	);
	*/
	println!("//////// 255 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("A/.."),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap(),
		root_without_public,
	);
	/*
	// useless with `ItemPath`
	println!("//////// 260 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A/../"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::IncorrectItemName {
			item_path: ItemPath::from("A/../"),
			error: String::from("`..` is not allowed")
		}
	);
	*/
	println!("//////// 265 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("A/../"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap(),
		root_without_public
	);
	/*
	// useless with `ItemPath`
	println!("//////// 270 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A/../AA"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::IncorrectItemName {
			item_path: ItemPath::from("A/../"),
			error: String::from("`..` is not allowed")
		}
	);
	*/
	println!("//////// 280 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A/A\0A"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::IncorrectItemName {
			item_path: ItemPath::from("A/A\0A"),
			error: format!("`{}` should not contains `\\0` character", "A\0A")
		}
	);
	println!("//////// 290 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("public/"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::CanNotBeListed {
			item_path: ItemPath::from("public/")
		},
	);
	println!("//////// 300 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("public/C/"),
			&Etag::from(""),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::CanNotBeListed {
			item_path: ItemPath::from("public/C/")
		}
	);

	////////////////////////////////////////////////////////////////////////////////////////////////

	println!("//////// 310 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from(""),
			&Etag::from("ANOTHER_ETAG"),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::NoIfMatch {
			item_path: ItemPath::from(""),
			search: Etag::from("ANOTHER_ETAG"),
			found: root.get_etag().clone()
		}
	);
	println!("//////// 320 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A/"),
			&Etag::from("ANOTHER_ETAG"),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::NoIfMatch {
			item_path: ItemPath::from("A/"),
			search: Etag::from("ANOTHER_ETAG"),
			found: A.get_etag().clone()
		}
	);
	println!("//////// 330 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A/AA"),
			&Etag::from("ANOTHER_ETAG"),
			&vec![],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::NoIfMatch {
			item_path: ItemPath::from("A/AA"),
			search: Etag::from("ANOTHER_ETAG"),
			found: AA.get_etag().clone()
		}
	);

	////////////////////////////////////////////////////////////////////////////////////////////////

	println!("//////// 340 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from(""),
			&Etag::from(""),
			&[&Etag::from("*")],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::IfNoneMatch {
			item_path: ItemPath::from(""),
			search: Etag::from("*"),
			found: root.get_etag().clone()
		}
	);
	println!("//////// 350 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A/"),
			&Etag::from(""),
			&[&Etag::from("*")],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::IfNoneMatch {
			item_path: ItemPath::from("A/"),
			search: Etag::from("*"),
			found: A.get_etag().clone()
		}
	);
	println!("//////// 360 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&[&Etag::from("*")],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::IfNoneMatch {
			item_path: ItemPath::from("A/AA"),
			search: Etag::from("*"),
			found: AA.get_etag().clone()
		}
	);

	////////////////////////////////////////////////////////////////////////////////////////////////

	println!("//////// 370 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from(""),
			&Etag::from(""),
			&[root.get_etag()],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::IfNoneMatch {
			item_path: ItemPath::from(""),
			search: root.get_etag().clone(),
			found: root.get_etag().clone()
		}
	);
	println!("//////// 380 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A/"),
			&Etag::from(""),
			&[A.get_etag()],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::IfNoneMatch {
			item_path: ItemPath::from("A/"),
			search: A.get_etag().clone(),
			found: A.get_etag().clone()
		}
	);
	println!("//////// 390 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&[AA.get_etag()],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::IfNoneMatch {
			item_path: ItemPath::from("A/AA"),
			search: AA.get_etag().clone(),
			found: AA.get_etag().clone()
		}
	);

	////////////////////////////////////////////////////////////////////////////////////////////////

	println!("//////// 400 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from(""),
			&Etag::from(""),
			&[],
			false
		)
		.unwrap(),
		root_without_public.empty_clone()
	);
	println!("//////// 410 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("A/"),
			&Etag::from(""),
			&[],
			false
		)
		.unwrap(),
		A.empty_clone()
	);
	println!("//////// 420 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&[],
			false
		)
		.unwrap(),
		AA.empty_clone()
	);
	println!("//////// 430 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("public/"),
			&Etag::from(""),
			&[],
			false
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::CanNotBeListed {
			item_path: ItemPath::from("public/")
		}
	);
	println!("//////// 440 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("public/C/"),
			&Etag::from(""),
			&[],
			false
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::CanNotBeListed {
			item_path: ItemPath::from("public/C/")
		}
	);
	println!("//////// 450 ////////");
	assert_eq!(
		get(
			&storage,
			prefix,
			&ItemPath::from("public/C/CA"),
			&Etag::from(""),
			&[],
			false
		)
		.unwrap(),
		CA.empty_clone()
	);
	println!("//////// 460 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("public/not_exists"),
			&Etag::from(""),
			&[],
			false
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::NotFound {
			item_path: ItemPath::from("public/not_exists")
		}
	);
	println!("//////// 470 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("public/not_exists/"),
			&Etag::from(""),
			&[],
			false
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::CanNotBeListed {
			item_path: ItemPath::from("public/not_exists/")
		}
	);

	////////////////////////////////////////////////////////////////////////////////////////////////

	println!("//////// 480 ////////");
	assert_eq!(
		*get(
			&storage,
			prefix,
			&ItemPath::from("A/.AA.itemdata.json"),
			&Etag::from(""),
			&[&Etag::from("*")],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::IsSystemFile
	);
}
