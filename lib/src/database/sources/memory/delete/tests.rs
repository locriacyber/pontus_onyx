#![allow(non_snake_case)]

use super::{delete, DeleteError};
use crate::item::{Etag, Item, ItemPath};

fn build_test_db() -> (Item, Etag, Etag, Etag, Etag, Etag, Etag) {
	let root = Item::new_folder(vec![
		(
			"A",
			Item::new_folder(vec![
				(
					"AA",
					Item::new_folder(vec![(
						"AAA",
						Item::new_folder(vec![("AAAA", Item::new_doc(b"AAAA", "text/plain"))]),
					)]),
				),
				("AB", Item::new_doc(b"AB", "text/plain")),
			]),
		),
		(
			"public",
			Item::new_folder(vec![(
				"C",
				Item::new_folder(vec![(
					"CC",
					Item::new_folder(vec![("CCC", Item::new_doc(b"CCC", "text/plain"))]),
				)]),
			)]),
		),
	]);

	if let Item::Folder {
		etag: root_etag,
		content: Some(content),
	} = &root
	{
		if let Item::Folder {
			etag: A_etag,
			content: Some(content),
		} = &**content.get("A").unwrap()
		{
			let AB_etag = if let Item::Document { etag, .. } = &**content.get("AB").unwrap() {
				etag
			} else {
				panic!();
			};

			if let Item::Folder {
				etag: AA_etag,
				content: Some(content),
			} = &**content.get("AA").unwrap()
			{
				if let Item::Folder {
					etag: AAA_etag,
					content: Some(content),
				} = &**content.get("AAA").unwrap()
				{
					if let Item::Document {
						etag: AAAA_etag, ..
					} = &**content.get("AAAA").unwrap()
					{
						return (
							root.clone(),
							root_etag.clone(),
							A_etag.clone(),
							AA_etag.clone(),
							AAA_etag.clone(),
							AAAA_etag.clone(),
							AB_etag.clone(),
						);
					} else {
						panic!();
					}
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	} else {
		panic!();
	}
}

#[test]
fn simple_delete_on_not_existing() {
	let mut root = Item::new_folder(vec![]);
	let root_etag = root.get_etag().clone();

	assert_eq!(
		*delete(&mut root, &ItemPath::from("A/AA/AAA/AAAA"), &Etag::from(""),)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
		DeleteError::NotFound {
			item_path: ItemPath::from("A/")
		}
	);

	if let Item::Folder {
		etag,
		content: Some(content),
	} = root
	{
		assert_eq!(etag, root_etag);
		assert!(content.is_empty());
	} else {
		panic!();
	}
}

#[test]
fn simple_delete_on_existing() {
	let (mut root, root_etag, A_etag, _, _, AAAA_etag, AB_etag) = build_test_db();

	let old_AAAA_etag =
		delete(&mut root, &ItemPath::from("A/AA/AAA/AAAA"), &Etag::from("")).unwrap();

	assert_eq!(AAAA_etag, old_AAAA_etag);

	if let Item::Folder {
		etag,
		content: Some(content),
	} = root
	{
		assert_ne!(etag, root_etag);
		assert!(!content.is_empty());

		if let Item::Folder {
			etag,
			content: Some(content),
		} = &**content.get("A").unwrap()
		{
			assert_ne!(etag, &A_etag);
			assert!(!content.is_empty());

			assert_eq!(content.get("AA"), None);

			if let Item::Document { etag, .. } = &**content.get("AB").unwrap() {
				assert_eq!(etag, &AB_etag);
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	} else {
		panic!();
	}
}

#[test]
fn does_not_works_for_folders() {
	let (mut root, root_etag, A_etag, AA_etag, AAA_etag, AAAA_etag, _) = build_test_db();

	assert_eq!(
		*delete(&mut root, &ItemPath::from("A/AA/"), &Etag::from(""),)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
		DeleteError::DoesNotWorksForFolders,
	);

	if let Item::Folder {
		etag,
		content: Some(content),
	} = &root
	{
		assert_eq!(etag, &root_etag);
		if let Item::Folder {
			etag,
			content: Some(content),
		} = &**content.get("A").unwrap()
		{
			assert_eq!(etag, &A_etag);
			if let Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("AA").unwrap()
			{
				assert_eq!(etag, &AA_etag);
				if let Item::Folder {
					etag,
					content: Some(content),
				} = &**content.get("AAA").unwrap()
				{
					assert_eq!(etag, &AAA_etag);
					if let Item::Document { etag, .. } = &**content.get("AAAA").unwrap() {
						assert_eq!(etag, &AAAA_etag);
					} else {
						panic!();
					}
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	} else {
		panic!();
	}
}

#[test]
fn delete_with_if_match_not_found() {
	let (mut root, root_etag, A_etag, AA_etag, AAA_etag, AAAA_etag, _) = build_test_db();

	assert_eq!(
		*delete(
			&mut root,
			&ItemPath::from("A/AA/AAA/AAAA"),
			&Etag::from("OTHER_ETAG"),
		)
		.unwrap_err()
		.downcast::<DeleteError>()
		.unwrap(),
		DeleteError::NoIfMatch {
			item_path: ItemPath::from("A/AA/AAA/AAAA"),
			found: AAAA_etag.clone(),
			search: Etag::from("OTHER_ETAG")
		}
	);

	if let Item::Folder {
		etag,
		content: Some(content),
	} = &root
	{
		assert_eq!(etag, &root_etag);
		if let Item::Folder {
			etag,
			content: Some(content),
		} = &**content.get("A").unwrap()
		{
			assert_eq!(etag, &A_etag);
			if let Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("AA").unwrap()
			{
				assert_eq!(etag, &AA_etag);
				if let Item::Folder {
					etag,
					content: Some(content),
				} = &**content.get("AAA").unwrap()
				{
					assert_eq!(etag, &AAA_etag);
					if let Item::Document { etag, .. } = &**content.get("AAAA").unwrap() {
						assert_eq!(etag, &AAAA_etag);
					} else {
						panic!();
					}
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	} else {
		panic!();
	}
}

#[test]
fn delete_with_if_match_found() {
	let (mut root, root_etag, A_etag, _, _, AAAA_etag, _) = build_test_db();

	let old_AAAA_etag = delete(&mut root, &ItemPath::from("A/AA/AAA/AAAA"), &AAAA_etag).unwrap();

	assert_eq!(old_AAAA_etag, AAAA_etag);

	if let Item::Folder {
		etag,
		content: Some(content),
	} = &root
	{
		assert_ne!(etag, &root_etag);
		assert!(!content.is_empty());

		if let Item::Folder {
			etag,
			content: Some(content),
		} = &**content.get("A").unwrap()
		{
			assert_ne!(etag, &A_etag);
			assert!(!content.is_empty());

			assert_eq!(content.get("AA"), None);
			assert!(content.get("AB").is_some());
		} else {
			panic!();
		}
	} else {
		panic!();
	}
}

#[test]
fn delete_with_if_match_all() {
	let (mut root, root_etag, A_etag, _, _, AAAA_etag, _) = build_test_db();

	let old_AAAA_etag = delete(
		&mut root,
		&ItemPath::from("A/AA/AAA/AAAA"),
		&Etag::from("*"),
	)
	.unwrap();

	assert_eq!(old_AAAA_etag, AAAA_etag);

	if let Item::Folder {
		etag,
		content: Some(content),
	} = &root
	{
		assert_ne!(etag, &root_etag);
		assert!(!content.is_empty());

		if let Item::Folder {
			etag,
			content: Some(content),
		} = &**content.get("A").unwrap()
		{
			assert_ne!(etag, &A_etag);
			assert!(!content.is_empty());

			assert_eq!(content.get("AA"), None);
			assert!(content.get("AB").is_some());
		} else {
			panic!();
		}
	} else {
		panic!();
	}
}

#[test]
fn delete_with_existing_folder_conflict() {
	let (mut root, root_etag, A_etag, AA_etag, AAA_etag, _, _) = build_test_db();

	assert_eq!(
		*delete(&mut root, &ItemPath::from("A/AA"), &Etag::from(""),)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
		DeleteError::Conflict {
			item_path: ItemPath::from("A/AA/")
		}
	);

	if let Item::Folder {
		etag,
		content: Some(content),
	} = &root
	{
		assert_eq!(etag, &root_etag);
		assert!(!content.is_empty());

		if let Item::Folder {
			etag,
			content: Some(content),
		} = &**content.get("A").unwrap()
		{
			assert_eq!(etag, &A_etag);
			assert!(!content.is_empty());

			if let Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("AA").unwrap()
			{
				assert_eq!(etag, &AA_etag);
				assert!(!content.is_empty());

				if let Item::Folder {
					etag,
					content: Some(content),
				} = &**content.get("AAA").unwrap()
				{
					assert_eq!(etag, &AAA_etag);
					assert!(!content.is_empty());
				}
			}

			assert!(content.get("AB").is_some());
		} else {
			panic!();
		}
	} else {
		panic!();
	}
}

#[test]
fn delete_in_public() {
	let (mut root, root_etag, _, _, _, _, _) = build_test_db();

	delete(
		&mut root,
		&ItemPath::from("public/C/CC/CCC"),
		&Etag::from(""),
	)
	.unwrap();

	if let Item::Folder {
		etag,
		content: Some(content),
	} = &root
	{
		assert_ne!(etag, &root_etag);
		assert!(!content.is_empty());

		assert!(content.get("A").is_some());
		assert_eq!(content.get("public"), None);
	} else {
		panic!();
	}
}

#[test]
fn delete_in_incorrect_path() {
	let mut root = Item::new_folder(vec![]);
	let root_etag = root.get_etag().clone();

	assert_eq!(
		*delete(&mut root, &ItemPath::from("A/A\0A"), &Etag::from(""),)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
		DeleteError::IncorrectItemName {
			item_path: ItemPath::from("A/A\0A"),
			error: String::from("`A\0A` should not contains `\\0` character")
		}
	);

	if let Item::Folder {
		etag,
		content: Some(content),
	} = root
	{
		assert_eq!(etag, root_etag);
		assert!(content.is_empty());
	} else {
		panic!();
	}
}
