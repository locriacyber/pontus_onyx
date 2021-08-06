#![allow(non_snake_case)]

use super::{put, PutError};
use crate::item::{Etag, Item, ItemPath};

// TODO : test last_modified

fn build_test_db() -> (
	Item,
	crate::item::Etag,
	crate::item::Etag,
	crate::item::Etag,
) {
	let root = Item::new_folder(vec![(
		"A",
		Item::new_folder(vec![("AA", Item::new_doc(b"AA", "text/plain"))]),
	)]);

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
			if let Item::Document { etag: AA_etag, .. } = &**content.get("AA").unwrap() {
				return (
					root.clone(),
					root_etag.clone(),
					A_etag.clone(),
					AA_etag.clone(),
				);
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	} else {
		panic!();
	};
}

#[test]
fn simple_put_on_not_existing() {
	let mut root = Item::new_folder(vec![]);
	let root_etag = root.get_etag().clone();

	let AA_etag = put(
		&mut root,
		&ItemPath::from("AA"),
		&Etag::from(""),
		&[],
		Item::new_doc(b"AA", "text/plain"),
	)
	.unwrap();

	if let Item::Folder {
		etag,
		content: Some(content),
	} = root
	{
		assert_ne!(etag, root_etag);
		assert!(!content.is_empty());

		if let Item::Document {
			etag,
			content: Some(content),
			content_type,
			..
		} = &**content.get("AA").unwrap()
		{
			assert_eq!(etag, &AA_etag);
			assert_eq!(content, &b"AA".to_vec());
			assert_eq!(content_type, &crate::item::ContentType::from("text/plain"));
		}
	} else {
		panic!();
	};
}

#[test]
fn simple_put_on_existing() {
	let (mut root, root_etag, A_etag, old_AA_etag) = build_test_db();

	let AA_etag = put(
		&mut root,
		&ItemPath::from("A/AA"),
		&Etag::from(""),
		&[],
		Item::new_doc(b"AA2", "text/plain2"),
	)
	.unwrap();

	assert_ne!(old_AA_etag, AA_etag);

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

			if let Item::Document {
				etag,
				content: Some(content),
				content_type,
				..
			} = &**content.get("AA").unwrap()
			{
				assert_eq!(etag, &AA_etag);
				assert_eq!(content, &b"AA2".to_vec());
				assert_eq!(content_type, &crate::item::ContentType::from("text/plain2"));
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	} else {
		panic!();
	};
}

#[test]
fn content_not_changed() {
	let (mut root, root_etag, A_etag, AA_etag) = build_test_db();

	assert_eq!(
		*put(
			&mut root,
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

	if let Item::Folder {
		etag,
		content: Some(content),
	} = root
	{
		assert_eq!(etag, root_etag);
		assert!(!content.is_empty());

		if let Item::Folder {
			etag,
			content: Some(content),
		} = &**content.get("A").unwrap()
		{
			assert_eq!(etag, &A_etag);
			assert!(!content.is_empty());

			if let Item::Document {
				etag,
				content: Some(content),
				content_type,
				..
			} = &**content.get("AA").unwrap()
			{
				assert_eq!(etag, &AA_etag);
				assert_eq!(content, &b"AA".to_vec());
				assert_eq!(content_type, &crate::item::ContentType::from("text/plain"));
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	} else {
		panic!();
	};
}

#[test]
fn does_not_works_for_folders() {
	let mut root = Item::new_folder(vec![]);
	let root_etag = root.get_etag().clone();

	assert_eq!(
		*put(
			&mut root,
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
	if let Item::Folder {
		etag,
		content: Some(content),
	} = root
	{
		assert_eq!(etag, root_etag);
		assert!(content.is_empty());
	} else {
		panic!();
	};
}

#[test]
fn put_with_if_none_match_all_on_not_existing() {
	let mut root = Item::new_folder(vec![]);
	let root_etag = root.get_etag().clone();

	let AA_etag = put(
		&mut root,
		&ItemPath::from("A/AA"),
		&Etag::from(""),
		&[&Etag::from("*")],
		Item::new_doc(b"AA", "text/plain"),
	)
	.unwrap();

	if let Item::Folder {
		etag,
		content: Some(content),
	} = root
	{
		assert_ne!(etag, root_etag);
		assert!(!content.is_empty());

		if let Item::Folder {
			content: Some(content),
			..
		} = &**content.get("A").unwrap()
		{
			assert!(!content.is_empty());

			if let Item::Document {
				etag,
				content: Some(content),
				content_type,
				..
			} = &**content.get("AA").unwrap()
			{
				assert_eq!(etag, &AA_etag);
				assert_eq!(content, &b"AA".to_vec());
				assert_eq!(content_type, &crate::item::ContentType::from("text/plain"));
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	} else {
		panic!();
	};
}

#[test]
fn put_with_if_none_match_all_on_existing() {
	let (mut root, root_etag, A_etag, AA_etag) = build_test_db();

	assert_eq!(
		*put(
			&mut root,
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

	if let Item::Folder {
		etag,
		content: Some(content),
	} = root
	{
		assert_eq!(etag, root_etag);
		assert!(!content.is_empty());

		if let Item::Folder {
			etag,
			content: Some(content),
		} = &**content.get("A").unwrap()
		{
			assert_eq!(etag, &A_etag);
			assert!(!content.is_empty());

			if let Item::Document {
				etag,
				content: Some(content),
				content_type,
				..
			} = &**content.get("AA").unwrap()
			{
				assert_eq!(etag, &AA_etag);
				assert_eq!(content, &b"AA".to_vec());
				assert_eq!(content_type, &crate::item::ContentType::from("text/plain"));
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	} else {
		panic!();
	};
}

#[test]
fn put_with_if_match_not_found() {
	let (mut root, root_etag, A_etag, AA_etag) = build_test_db();

	assert_eq!(
		*put(
			&mut root,
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

	if let Item::Folder {
		etag,
		content: Some(content),
	} = root
	{
		assert_eq!(etag, root_etag);
		assert!(!content.is_empty());

		if let Item::Folder {
			etag,
			content: Some(content),
		} = &**content.get("A").unwrap()
		{
			assert_eq!(etag, &A_etag);
			assert!(!content.is_empty());

			if let Item::Document {
				etag,
				content: Some(content),
				content_type,
				..
			} = &**content.get("AA").unwrap()
			{
				assert_eq!(etag, &AA_etag);
				assert_eq!(content, &b"AA".to_vec());
				assert_eq!(content_type, &crate::item::ContentType::from("text/plain"));
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	} else {
		panic!();
	};
}

#[test]
fn put_with_if_match_found() {
	let (mut root, root_etag, A_etag, mut AA_etag) = build_test_db();

	AA_etag = put(
		&mut root,
		&ItemPath::from("A/AA"),
		&AA_etag,
		&[],
		Item::new_doc(b"AA2", "text/plain2"),
	)
	.unwrap();

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

			if let Item::Document {
				etag,
				content: Some(content),
				content_type,
				..
			} = &**content.get("AA").unwrap()
			{
				assert_eq!(etag, &AA_etag);
				assert_eq!(content, &b"AA2".to_vec());
				assert_eq!(content_type, &crate::item::ContentType::from("text/plain2"));
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	} else {
		panic!();
	};
}

#[test]
fn put_with_if_match_all() {
	let (mut root, root_etag, A_etag, old_AA_etag) = build_test_db();

	let AA_etag = put(
		&mut root,
		&ItemPath::from("A/AA"),
		&Etag::from("*"),
		&[],
		Item::new_doc(b"AA2", "text/plain2"),
	)
	.unwrap();

	assert_ne!(old_AA_etag, AA_etag);

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

			if let Item::Document {
				etag,
				content: Some(content),
				content_type,
				..
			} = &**content.get("AA").unwrap()
			{
				assert_eq!(etag, &AA_etag);
				assert_eq!(content, &b"AA2".to_vec());
				assert_eq!(content_type, &crate::item::ContentType::from("text/plain2"));
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	} else {
		panic!();
	};
}

#[test]
fn put_with_existing_document_conflict() {
	let (mut root, root_etag, A_etag, AA_etag) = build_test_db();

	assert_eq!(
		*put(
			&mut root,
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

	if let Item::Folder {
		etag,
		content: Some(content),
	} = root
	{
		assert_eq!(etag, root_etag);
		assert!(!content.is_empty());

		if let Item::Folder {
			etag,
			content: Some(content),
		} = &**content.get("A").unwrap()
		{
			assert_eq!(etag, &A_etag);
			assert!(!content.is_empty());

			if let Item::Document {
				etag,
				content: Some(content),
				content_type,
				..
			} = &**content.get("AA").unwrap()
			{
				assert_eq!(etag, &AA_etag);
				assert_eq!(content, &b"AA".to_vec());
				assert_eq!(content_type, &crate::item::ContentType::from("text/plain"));
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	} else {
		panic!();
	};
}

#[test]
fn put_with_existing_folder_conflict() {
	let (mut root, root_etag, A_etag, AA_etag) = build_test_db();

	assert_eq!(
		*put(
			&mut root,
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

	if let Item::Folder {
		etag,
		content: Some(content),
	} = root
	{
		assert_eq!(etag, root_etag);
		assert!(!content.is_empty());

		if let Item::Folder {
			etag,
			content: Some(content),
		} = &**content.get("A").unwrap()
		{
			assert_eq!(etag, &A_etag);
			assert!(!content.is_empty());

			if let Item::Document {
				etag,
				content: Some(content),
				content_type,
				..
			} = &**content.get("AA").unwrap()
			{
				assert_eq!(etag, &AA_etag);
				assert_eq!(content, &b"AA".to_vec());
				assert_eq!(content_type, &crate::item::ContentType::from("text/plain"));
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	} else {
		panic!();
	};
}

#[test]
fn put_in_public() {
	let mut root = Item::new_folder(vec![]);
	let root_etag = root.get_etag().clone();

	let AA_etag = put(
		&mut root,
		&ItemPath::from("public/A/AA"),
		&Etag::from(""),
		&[],
		Item::new_doc(b"AA", "text/plain"),
	)
	.unwrap();

	if let Item::Folder {
		etag,
		content: Some(content),
	} = root
	{
		assert_ne!(etag, root_etag);
		assert!(!content.is_empty());

		if let Item::Folder {
			content: Some(content),
			..
		} = &**content.get("public").unwrap()
		{
			assert!(!content.is_empty());
			if let Item::Folder {
				content: Some(content),
				..
			} = &**content.get("A").unwrap()
			{
				assert!(!content.is_empty());

				if let Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA".to_vec());
					assert_eq!(content_type, &crate::item::ContentType::from("text/plain"));
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	} else {
		panic!();
	};
}

#[test]
fn put_in_incorrect_path() {
	let mut root = Item::new_folder(vec![]);
	let root_etag = root.get_etag().clone();

	assert_eq!(
		*put(
			&mut root,
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

	if let Item::Folder {
		etag,
		content: Some(content),
	} = root
	{
		assert_eq!(etag, root_etag);
		assert!(content.is_empty());
	} else {
		panic!();
	};
}
