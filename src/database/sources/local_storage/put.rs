pub fn put(
	_storage: &dyn super::Storage,
	_prefix: &str,
	_path: &std::path::Path,
	_if_match: &crate::Etag,
	_if_none_match: &[&crate::Etag],
	_item: crate::Item,
) -> crate::database::PutResult {
	todo!()
}

#[derive(Debug, PartialEq, Eq)]
pub enum PutError {
	GetError(super::GetError),
	DoesNotWorksForFolders,
	ContentNotChanged,
}
impl std::fmt::Display for PutError {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		todo!()
	}
}
impl std::error::Error for PutError {}
#[cfg(feature = "server_bin")]
impl crate::database::Error for PutError {
	fn to_response(&self, _origin: &str, _should_have_body: bool) -> actix_web::HttpResponse {
		todo!()
	}
}

/*
#[cfg(test)]
mod tests {
	#![allow(non_snake_case)]

	use super::{super::LocalStorageMock, super::Storage, put, PutError};

	// TODO : test last_modified

	fn build_test_db() -> (
		LocalStorageMock,
		String,
		crate::Etag,
		crate::Etag,
		crate::Etag,
	) {
		let AA = crate::Item::new_doc(b"AA", "text/plain");
		let A = crate::Item::new_folder(vec![("AA", AA.clone())]);
		let root = crate::Item::new_folder(vec![("A", A.clone())]);

		////////////////////////////////////////////////////////////////////////////////////////////////

		let prefix = "pontus_onyx_put_test";

		let storage = LocalStorageMock::new();

		storage.set_item(&format!("{}/", prefix), "").unwrap();

		storage
			.set_item(
				&format!("{}/.folder.itemdata.toml", prefix),
				&serde_json::to_string(&crate::DataFolder {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: root.get_etag().clone(),
				})
				.unwrap(),
			)
			.unwrap();

		storage.set_item(&format!("{}/A/", prefix), "").unwrap();

		storage
			.set_item(
				&format!("{}/A/.folder.itemdata.toml", prefix),
				&serde_json::to_string(&crate::DataFolder {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: A.get_etag().clone(),
				})
				.unwrap(),
			)
			.unwrap();

		if let crate::Item::Document {
			content: Some(content),
			etag: AA_etag,
			content_type: AA_content_type,
			last_modified: AA_last_modified,
		} = AA.clone()
		{
			storage
				.set_item(
					&format!("{}/A/.AA.itemdata.toml", prefix),
					&serde_json::to_string(&crate::DataDocument {
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

		let mut root = crate::Item::new_folder(vec![]);
		let root_etag = root.get_etag().clone();

		let AA_etag = put(
			&storage,
			&prefix,
			&std::path::Path::new("AA"),
			&crate::Etag::from(""),
			&[],
			crate::Item::new_doc(b"AA", "text/plain"),
		)
		.unwrap();

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Document {
				etag,
				content: Some(content),
				content_type,
				..
			} = &**content.get("AA").unwrap()
			{
				assert_eq!(etag, &AA_etag);
				assert_eq!(content, &b"AA".to_vec());
				assert_eq!(content_type, &crate::ContentType::from("text/plain"));
			}
		} else {
			panic!();
		};
	}

	#[test]
	fn simple_put_on_existing() {
		let (storage, prefix, root_etag, A_etag, old_AA_etag) = build_test_db();

		let AA_etag = put(
			&storage,
			&prefix,
			&std::path::Path::new("A/AA"),
			&crate::Etag::from(""),
			&[],
			crate::Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap();

		assert_ne!(old_AA_etag, AA_etag);

		/*
		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_ne!(etag, &A_etag);
				assert!(!content.is_empty());

				if let crate::Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA2".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain2"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
		*/

		todo!()
	}

	#[test]
	fn content_not_changed() {
		let (storage, prefix, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			*put(
				&storage,
				&prefix,
				&std::path::Path::new("A/AA"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"AA", "text/plain")
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::ContentNotChanged
		);

		/*
		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				assert!(!content.is_empty());

				if let crate::Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
		*/

		todo!()
	}

	#[test]
	fn does_not_works_for_folders() {
		let prefix = "pontus_onyx_get_test";
		let storage = LocalStorageMock::new();

		assert_eq!(
			*put(
				&storage,
				&prefix,
				&std::path::Path::new(""),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_folder(vec![])
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::DoesNotWorksForFolders
		);

		/*
		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(content.is_empty());
		} else {
			panic!();
		};
		*/

		todo!()
	}

	#[test]
	fn put_with_if_none_match_all_on_not_existing() {
		let storage = LocalStorageMock::new();
		let prefix = String::from("pontus_onyx_put_test");

		let AA_etag = put(
			&storage,
			&prefix,
			&std::path::Path::new("A/AA"),
			&crate::Etag::from(""),
			&[&crate::Etag::from("*")],
			crate::Item::new_doc(b"AA", "text/plain"),
		)
		.unwrap();

		/*
		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				content: Some(content),
				..
			} = &**content.get("A").unwrap()
			{
				assert!(!content.is_empty());

				if let crate::Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
		*/

		todo!()
	}

	#[test]
	fn put_with_if_none_match_all_on_existing() {
		let (storage, prefix, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			*put(
				&storage,
				&prefix,
				&std::path::Path::new("A/AA"),
				&crate::Etag::from(""),
				&[&crate::Etag::from("*")],
				crate::Item::new_doc(b"AA2", "text/plain2"),
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(super::super::GetError::IfNoneMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				found: AA_etag.clone(),
				search: crate::Etag::from("*"),
			})
		);

		/*
		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				assert!(!content.is_empty());

				if let crate::Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
		*/

		todo!()
	}

	#[test]
	fn put_with_if_match_not_found() {
		let (storage, prefix, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			*put(
				&storage,
				&prefix,
				&std::path::Path::new("A/AA"),
				&crate::Etag::from("ANOTHER_ETAG"),
				&[],
				crate::Item::new_doc(b"AA2", "text/plain2"),
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(super::super::GetError::NoIfMatch {
				item_path: std::path::PathBuf::from("A/AA"),
				found: AA_etag.clone(),
				search: crate::Etag::from("ANOTHER_ETAG"),
			})
		);

		/*
		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				assert!(!content.is_empty());

				if let crate::Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
		*/

		todo!()
	}

	#[test]
	fn put_with_if_match_found() {
		let (storage, prefix, root_etag, A_etag, mut AA_etag) = build_test_db();

		AA_etag = put(
			&storage,
			&prefix,
			&std::path::Path::new("A/AA"),
			&AA_etag,
			&[],
			crate::Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap();

		/*
		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_ne!(etag, &A_etag);
				assert!(!content.is_empty());

				if let crate::Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA2".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain2"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
		*/

		todo!()
	}

	#[test]
	fn put_with_if_match_all() {
		let (storage, prefix, root_etag, A_etag, old_AA_etag) = build_test_db();

		let AA_etag = put(
			&storage,
			&prefix,
			&std::path::Path::new("A/AA"),
			&crate::Etag::from("*"),
			&[],
			crate::Item::new_doc(b"AA2", "text/plain2"),
		)
		.unwrap();

		assert_ne!(old_AA_etag, AA_etag);

		/*
		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_ne!(etag, &A_etag);
				assert!(!content.is_empty());

				if let crate::Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA2".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain2"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
		*/

		todo!()
	}

	#[test]
	fn put_with_existing_document_conflict() {
		let (storage, prefix, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			*put(
				&storage,
				&prefix,
				&std::path::Path::new("A/AA/AAA"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"AAA", "text/plain")
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(super::super::GetError::Conflict {
				item_path: std::path::PathBuf::from("A/AA")
			})
		);

		/*
		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				assert!(!content.is_empty());

				if let crate::Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
		*/

		todo!()
	}

	#[test]
	fn put_with_existing_folder_conflict() {
		let (storage, prefix, root_etag, A_etag, AA_etag) = build_test_db();

		assert_eq!(
			*put(
				&storage,
				&prefix,
				&std::path::Path::new("A"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"A", "text/plain")
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(super::super::GetError::Conflict {
				item_path: std::path::PathBuf::from("A")
			})
		);

		/*
		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				assert!(!content.is_empty());

				if let crate::Item::Document {
					etag,
					content: Some(content),
					content_type,
					..
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					assert_eq!(content, &b"AA".to_vec());
					assert_eq!(content_type, &crate::ContentType::from("text/plain"));
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
		*/

		todo!()
	}

	#[test]
	fn put_in_public() {
		let storage = LocalStorageMock::new();
		let prefix = String::from("pontus_onyx_put_test");

		let AA_etag = put(
			&storage,
			&prefix,
			&std::path::Path::new("public/A/AA"),
			&crate::Etag::from(""),
			&[],
			crate::Item::new_doc(b"AA", "text/plain"),
		)
		.unwrap();

		/*
		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				content: Some(content),
				..
			} = &**content.get("public").unwrap()
			{
				assert!(!content.is_empty());
				if let crate::Item::Folder {
					content: Some(content),
					..
				} = &**content.get("A").unwrap()
				{
					assert!(!content.is_empty());

					if let crate::Item::Document {
						etag,
						content: Some(content),
						content_type,
						..
					} = &**content.get("AA").unwrap()
					{
						assert_eq!(etag, &AA_etag);
						assert_eq!(content, &b"AA".to_vec());
						assert_eq!(content_type, &crate::ContentType::from("text/plain"));
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
		*/

		todo!()
	}

	#[test]
	fn put_in_incorrect_path() {
		let storage = LocalStorageMock::new();
		let prefix = String::from("pontus_onyx_put_test");

		assert_eq!(
			*put(
				&storage,
				&prefix,
				&std::path::Path::new("A/../AA"),
				&crate::Etag::from(""),
				&[],
				crate::Item::new_doc(b"AA", "text/plain"),
			)
			.unwrap_err()
			.downcast::<PutError>()
			.unwrap(),
			PutError::GetError(super::super::GetError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/../"),
				error: String::from("`..` is not allowed")
			})
		);

		/*
		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(content.is_empty());
		} else {
			panic!();
		};
		*/

		todo!()
	}
}
*/
