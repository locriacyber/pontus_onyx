#![allow(non_snake_case)]

use super::{get, GetError};
use crate::item::{DataDocument, DataFolder, Etag, Item, ItemPath};
use std::convert::TryFrom;

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

	let tmp_folder = tempfile::tempdir().unwrap();
	let tmp_folder_path = tmp_folder.path().to_path_buf();

	let root_path = tmp_folder_path.clone();

	let A_path = tmp_folder_path.join("A");
	let B_path = tmp_folder_path.join("B");
	let public_path = tmp_folder_path.join("public");
	let C_path = public_path.join("C");
	let AA_path = A_path.join("AA");
	let AB_path = A_path.join("AB");
	let AC_path = A_path.join("AC");
	let BA_path = B_path.join("BA");
	let BB_path = B_path.join("BB");
	let CA_path = C_path.join("CA");

	std::fs::create_dir_all(&A_path).unwrap();
	std::fs::create_dir_all(&B_path).unwrap();
	std::fs::create_dir_all(&C_path).unwrap();

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

	let C_data_path = C_path.join(".folder.itemdata.toml");
	std::fs::write(
		C_data_path,
		toml::to_string(&DataFolder {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: C.get_etag().clone(),
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

	let AB_data_path = A_path.join(".AB.itemdata.toml");
	std::fs::write(
		AB_data_path,
		toml::to_string(&DataDocument::try_from(AB.clone()).unwrap()).unwrap(),
	)
	.unwrap();

	let AC_data_path = A_path.join(".AC.itemdata.toml");
	std::fs::write(
		AC_data_path,
		toml::to_string(&DataDocument::try_from(AC.clone()).unwrap()).unwrap(),
	)
	.unwrap();

	let BA_data_path = B_path.join(".BA.itemdata.toml");
	std::fs::write(
		BA_data_path,
		toml::to_string(&DataDocument::try_from(BA.clone()).unwrap()).unwrap(),
	)
	.unwrap();

	let BB_data_path = B_path.join(".BB.itemdata.toml");
	std::fs::write(
		BB_data_path,
		toml::to_string(&&DataDocument::try_from(BB.clone()).unwrap()).unwrap(),
	)
	.unwrap();

	let CA_data_path = C_path.join(".CA.itemdata.toml");
	std::fs::write(
		CA_data_path,
		toml::to_string(&&DataDocument::try_from(CA.clone()).unwrap()).unwrap(),
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

	if let Item::Document {
		content: Some(content),
		..
	} = &AB
	{
		std::fs::write(AB_path, content).unwrap();
	} else {
		panic!()
	}

	if let Item::Document {
		content: Some(content),
		..
	} = &AC
	{
		std::fs::write(AC_path, content).unwrap();
	} else {
		panic!()
	}

	if let Item::Document {
		content: Some(content),
		..
	} = &BA
	{
		std::fs::write(BA_path, content).unwrap();
	} else {
		panic!()
	}

	if let Item::Document {
		content: Some(content),
		..
	} = &BB
	{
		std::fs::write(BB_path, content).unwrap();
	} else {
		panic!()
	}

	if let Item::Document {
		content: Some(content),
		..
	} = &CA
	{
		std::fs::write(CA_path, content).unwrap();
	} else {
		panic!()
	}

	////////////////////////////////////////////////////////////////////////////////////////////////

	assert_eq!(
		get(&root_path, &ItemPath::from(""), &Etag::from(""), &[], true).unwrap(),
		root_without_public.clone()
	);
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("A/"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap(),
		A.clone()
	);
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap(),
		AA.clone()
	);
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("A/AB"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap(),
		AB
	);
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("A/AC"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap(),
		AC
	);
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("B/"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap(),
		B
	);
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("B/BA"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap(),
		BA
	);
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("B/BB"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap(),
		BB
	);
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("public/C/CA"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap(),
		CA
	);

	////////////////////////////////////////////////////////////////////////////////////////////////

	assert_eq!(
		get(&root_path, &ItemPath::from(""), root.get_etag(), &[], true).unwrap(),
		root_without_public.clone()
	);
	assert_eq!(
		get(&root_path, &ItemPath::from("A/"), A.get_etag(), &[], true).unwrap(),
		A.clone()
	);
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("A/AA"),
			AA.get_etag(),
			&[],
			true
		)
		.unwrap(),
		AA.clone()
	);

	////////////////////////////////////////////////////////////////////////////////////////////////

	assert_eq!(
		get(
			&root_path,
			&ItemPath::from(""),
			&Etag::from(""),
			&[&Etag::from("ANOTHER_ETAG")],
			true
		)
		.unwrap(),
		root_without_public.clone()
	);
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("A/"),
			&Etag::from(""),
			&[&Etag::from("ANOTHER_ETAG")],
			true
		)
		.unwrap(),
		A.clone()
	);
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&[&Etag::from("ANOTHER_ETAG")],
			true
		)
		.unwrap(),
		AA.clone()
	);

	////////////////////////////////////////////////////////////////////////////////////////////////

	assert_eq!(
		*get(&root_path, &ItemPath::from("A"), &Etag::from(""), &[], true)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
		GetError::Conflict {
			item_path: ItemPath::from("A/")
		}
	);
	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("A/AA/"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::Conflict {
			item_path: ItemPath::from("A/AA")
		}
	);
	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("A/AC/not_exists"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::Conflict {
			item_path: ItemPath::from("A/AC")
		}
	);
	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("A/not_exists"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::NotFound {
			item_path: ItemPath::from("A/not_exists")
		}
	);
	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("A/not_exists/nested"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::NotFound {
			item_path: ItemPath::from("A/not_exists/")
		}
	);
	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("B/not_exists"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::NotFound {
			item_path: ItemPath::from("B/not_exists")
		}
	);
	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("not_exists/"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::NotFound {
			item_path: ItemPath::from("not_exists/")
		}
	);
	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("not_exists"),
			&Etag::from(""),
			&[],
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
	assert_eq!(
		*get(&root_path, &ItemPath::from("."), &Etag::from(""), &[], true)
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
		GetError::IncorrectItemName {
			item_path: ItemPath::from("."),
			error: String::from("`.` is not allowed")
		}
	);
	*/
	assert_eq!(
		get(&root_path, &ItemPath::from("."), &Etag::from(""), &[], true).unwrap(),
		root_without_public.clone()
	);
	/*
	// useless with `ItemPath`
	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("A/.."),
			&Etag::from(""),
			&[],
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
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("A/.."),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap(),
		root_without_public.clone()
	);
	/*
	// useless with `ItemPath`
	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("A/../"),
			&Etag::from(""),
			&[],
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
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("A/../"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap(),
		root_without_public.clone()
	);
	/*
	// useless with `ItemPath` :
	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("A/../AA"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::IncorrectItemName {
			item_path: std::path::PathBuf::from("A/../"),
			error: String::from("`..` is not allowed")
		}
	);
	*/
	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("A/A\0A"),
			&Etag::from(""),
			&[],
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
	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("public/"),
			&Etag::from(""),
			&[],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::CanNotBeListed {
			item_path: ItemPath::from("public/")
		},
	);
	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("public/C/"),
			&Etag::from(""),
			&[],
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

	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from(""),
			&Etag::from("ANOTHER_ETAG"),
			&[],
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
	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("A/"),
			&Etag::from("ANOTHER_ETAG"),
			&[],
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
	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("A/AA"),
			&Etag::from("ANOTHER_ETAG"),
			&[],
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

	assert_eq!(
		*get(
			&root_path,
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
	assert_eq!(
		*get(
			&root_path,
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
	assert_eq!(
		*get(
			&root_path,
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

	assert_eq!(
		*get(
			&root_path,
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
	assert_eq!(
		*get(
			&root_path,
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
	assert_eq!(
		*get(
			&root_path,
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

	assert_eq!(
		get(&root_path, &ItemPath::from(""), &Etag::from(""), &[], false).unwrap(),
		root_without_public.empty_clone()
	);
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("A/"),
			&Etag::from(""),
			&[],
			false
		)
		.unwrap(),
		A.empty_clone()
	);
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&[],
			false
		)
		.unwrap(),
		AA.empty_clone()
	);
	assert_eq!(
		*get(
			&root_path,
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
	assert_eq!(
		*get(
			&root_path,
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
	assert_eq!(
		get(
			&root_path,
			&ItemPath::from("public/C/CA"),
			&Etag::from(""),
			&[],
			false
		)
		.unwrap(),
		CA.empty_clone()
	);
	assert_eq!(
		*get(
			&root_path,
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
	assert_eq!(
		*get(
			&root_path,
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

	assert_eq!(
		*get(
			&root_path,
			&ItemPath::from("A/.AA.itemdata.toml"),
			&Etag::from(""),
			&[&Etag::from("*")],
			true
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::IsSystemFile
	);

	////////////////////////////////////////////////////////////////////////////////////////////////

	tmp_folder.close().unwrap();
}
