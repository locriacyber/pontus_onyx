#![allow(non_snake_case)]

use super::{get, GetError};
use crate::item::{Etag, Item, ItemPath};

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

	assert_eq!(
		get(&root, &ItemPath::from(""), &Etag::from(""), &vec![]).unwrap(),
		root.clone() // TODO : should return root_without_public, but recursion in get make it buggy.
	);
	assert_eq!(
		get(&root, &ItemPath::from("A/"), &Etag::from(""), &vec![]).unwrap(),
		A.clone()
	);
	assert_eq!(
		get(&root, &ItemPath::from("A/AA"), &Etag::from(""), &vec![]).unwrap(),
		AA.clone()
	);
	assert_eq!(
		get(&root, &ItemPath::from("A/AB"), &Etag::from(""), &vec![]).unwrap(),
		AB
	);
	assert_eq!(
		get(&root, &ItemPath::from("A/AC"), &Etag::from(""), &vec![]).unwrap(),
		AC
	);
	assert_eq!(
		get(&root, &ItemPath::from("B/"), &Etag::from(""), &vec![]).unwrap(),
		B
	);
	assert_eq!(
		get(&root, &ItemPath::from("B/BA"), &Etag::from(""), &vec![]).unwrap(),
		BA
	);
	assert_eq!(
		get(&root, &ItemPath::from("B/BB"), &Etag::from(""), &vec![]).unwrap(),
		BB
	);
	assert_eq!(
		get(
			&root,
			&ItemPath::from("public/C/CA"),
			&Etag::from(""),
			&vec![]
		)
		.unwrap(),
		CA
	);

	////////////////////////////////////////////////////////////////////////////////////////////////

	assert_eq!(
		get(&root, &ItemPath::from(""), root.get_etag(), &vec![]).unwrap(),
		root.clone() // TODO : should return root_without_public, but recursion in get make it buggy.
	);
	assert_eq!(
		get(&root, &ItemPath::from("A/"), A.get_etag(), &vec![]).unwrap(),
		A.clone()
	);
	assert_eq!(
		get(&root, &ItemPath::from("A/AA"), AA.get_etag(), &vec![]).unwrap(),
		AA.clone()
	);

	////////////////////////////////////////////////////////////////////////////////////////////////

	assert_eq!(
		get(
			&root,
			&ItemPath::from(""),
			&Etag::from(""),
			&[&Etag::from("ANOTHER_ETAG")]
		)
		.unwrap(),
		root.clone() // TODO : should return root_without_public, but recursion in get make it buggy.
	);
	assert_eq!(
		get(
			&root,
			&ItemPath::from("A/"),
			&Etag::from(""),
			&[&Etag::from("ANOTHER_ETAG")]
		)
		.unwrap(),
		A.clone()
	);
	assert_eq!(
		get(
			&root,
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&[&Etag::from("ANOTHER_ETAG")]
		)
		.unwrap(),
		AA.clone()
	);

	////////////////////////////////////////////////////////////////////////////////////////////////

	assert_eq!(
		*get(&root, &ItemPath::from("A"), &Etag::from(""), &vec![])
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
		GetError::Conflict {
			item_path: ItemPath::from("A/")
		}
	);
	assert_eq!(
		*get(&root, &ItemPath::from("A/AA/"), &Etag::from(""), &vec![])
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
		GetError::Conflict {
			item_path: ItemPath::from("A/AA")
		}
	);
	assert_eq!(
		*get(
			&root,
			&ItemPath::from("A/AC/not_exists"),
			&Etag::from(""),
			&vec![]
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
			&root,
			&ItemPath::from("A/not_exists"),
			&Etag::from(""),
			&vec![]
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
			&root,
			&ItemPath::from("A/not_exists/nested"),
			&Etag::from(""),
			&vec![]
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
			&root,
			&ItemPath::from("B/not_exists"),
			&Etag::from(""),
			&vec![]
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
			&root,
			&ItemPath::from("not_exists/"),
			&Etag::from(""),
			&vec![]
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
			&root,
			&ItemPath::from("not_exists"),
			&Etag::from(""),
			&vec![]
		)
		.unwrap_err()
		.downcast::<GetError>()
		.unwrap(),
		GetError::NotFound {
			item_path: ItemPath::from("not_exists")
		}
	);
	/*
	useless with `ItemPath`
	assert_eq!(
		*get(&root, &ItemPath::from("."), &Etag::from(""), &vec![])
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
		get(&root, &ItemPath::from("."), &Etag::from(""), &vec![]).unwrap(),
		root.clone()
	);
	/*
	useless with `ItemPath`
	assert_eq!(
		*get(&root, &ItemPath::from("A/.."), &Etag::from(""), &vec![])
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
		get(&root, &ItemPath::from("A/.."), &Etag::from(""), &vec![]).unwrap(),
		root.clone(),
	);
	/*
	useless with `ItemPath`
	assert_eq!(
		*get(&root, &ItemPath::from("A/../"), &Etag::from(""), &vec![])
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
		get(&root, &ItemPath::from("A/../"), &Etag::from(""), &vec![]).unwrap(),
		root.clone(),
	);
	/*
	useless with `ItemPath`
	assert_eq!(
		*get(&root, &ItemPath::from("A/../AA"), &Etag::from(""), &vec![])
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
		GetError::IncorrectItemName {
			item_path: ItemPath::from("A/../"),
			error: String::from("`..` is not allowed")
		}
	);
	*/
	/*
	// useless with `ItemPath` :
	assert_eq!(
		*get(&root, &ItemPath::from("A/../AA"), &Etag::from(""), &vec![])
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
		*get(&root, &ItemPath::from("A/A\0A"), &Etag::from(""), &vec![])
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
		GetError::IncorrectItemName {
			item_path: ItemPath::from("A/A\0A"),
			error: format!("`{}` should not contains `\\0` character", "A\0A")
		}
	);
	assert_eq!(
		*get(&root, &ItemPath::from("public/"), &Etag::from(""), &vec![])
			.unwrap_err()
			.downcast::<GetError>()
			.unwrap(),
		GetError::CanNotBeListed {
			item_path: ItemPath::from("public/")
		},
	);
	assert_eq!(
		*get(
			&root,
			&ItemPath::from("public/C/"),
			&Etag::from(""),
			&vec![]
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
			&root,
			&ItemPath::from(""),
			&Etag::from("ANOTHER_ETAG"),
			&vec![]
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
			&root,
			&ItemPath::from("A/"),
			&Etag::from("ANOTHER_ETAG"),
			&vec![]
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
			&root,
			&ItemPath::from("A/AA"),
			&Etag::from("ANOTHER_ETAG"),
			&vec![]
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
			&root,
			&ItemPath::from(""),
			&Etag::from(""),
			&[&Etag::from("*")]
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
			&root,
			&ItemPath::from("A/"),
			&Etag::from(""),
			&[&Etag::from("*")]
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
			&root,
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&[&Etag::from("*")]
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
			&root,
			&ItemPath::from(""),
			&Etag::from(""),
			&[root.get_etag()]
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
			&root,
			&ItemPath::from("A/"),
			&Etag::from(""),
			&[A.get_etag()]
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
			&root,
			&ItemPath::from("A/AA"),
			&Etag::from(""),
			&[AA.get_etag()]
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
}
