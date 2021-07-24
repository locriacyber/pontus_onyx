pub fn delete(
	_storage: &dyn super::Storage,
	_prefix: &str,
	_path: &std::path::Path,
	_if_match: &crate::Etag,
) -> Result<crate::Etag, Box<dyn std::error::Error>> {
	todo!()
}

#[derive(Debug, PartialEq)]
pub enum DeleteError {
	GetError(super::GetError),
	DoesNotWorksForFolders,
}
impl std::fmt::Display for DeleteError {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		todo!()
	}
}
impl std::error::Error for DeleteError {}
#[cfg(feature = "server_bin")]
impl crate::database::Error for DeleteError {
	fn to_response(&self, _origin: &str, _should_have_body: bool) -> actix_web::HttpResponse {
		todo!()
	}
}

/*
#[cfg(test)]
mod tests {
	#![allow(non_snake_case)]

	use super::{super::GetError, super::LocalStorageMock, super::Storage, delete, DeleteError};

	// TODO : test last_modified

	fn build_test_db() -> (
		LocalStorageMock,
		String,
		crate::Etag,
		crate::Etag,
		crate::Etag,
		crate::Etag,
		crate::Etag,
		crate::Etag,
		crate::Etag,
		crate::Etag,
	) {
		let AAA = crate::Item::new_doc(b"AAA", "text/plain");
		let AA = crate::Item::new_folder(vec![("AAA", AAA.clone())]);
		let AB = crate::Item::new_doc(b"AB", "text/plain");
		let A = crate::Item::new_folder(vec![("AA", AA.clone()), ("AB", AB.clone())]);

		let BA = crate::Item::new_doc(b"BA", "text/plain");
		let B = crate::Item::new_folder(vec![("BA", BA.clone())]);
		let public = crate::Item::new_folder(vec![("B", B.clone())]);

		let root = crate::Item::new_folder(vec![("A", A.clone()), ("public", public.clone())]);

		////////////////////////////////////////////////////////////////////////////////////////////////

		let prefix = String::from("pontus_onyx_delete_test");

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

		if let crate::Item::Document {
			content: Some(content),
			etag: AB_etag,
			content_type: AB_content_type,
			last_modified: AB_last_modified,
		} = AB.clone()
		{
			storage
				.set_item(
					&format!("{}/A/.AB.itemdata.toml", prefix),
					&serde_json::to_string(&crate::DataDocument {
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

		storage.set_item(&format!("{}/B/", prefix), "").unwrap();

		storage
			.set_item(
				&format!("{}/B/.folder.itemdata.toml", prefix),
				&serde_json::to_string(&crate::DataFolder {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: B.get_etag().clone(),
				})
				.unwrap(),
			)
			.unwrap();

		if let crate::Item::Document {
			content: Some(content),
			etag: BA_etag,
			content_type: BA_content_type,
			last_modified: BA_last_modified,
		} = BA.clone()
		{
			storage
				.set_item(
					&format!("{}/B/.BA.itemdata.toml", prefix),
					&serde_json::to_string(&crate::DataDocument {
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

		storage
			.set_item(&format!("{}/public/", prefix), "")
			.unwrap();

		storage
			.set_item(
				&format!("{}/public/.folder.itemdata.toml", prefix),
				&serde_json::to_string(&crate::DataFolder {
					datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
					etag: public.get_etag().clone(),
				})
				.unwrap(),
			)
			.unwrap();

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

		assert_eq!(
			*delete(
				&storage,
				&prefix,
				&std::path::Path::new("A/AA/AAA"),
				&crate::Etag::from(""),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::GetError(GetError::NotFound {
				item_path: std::path::PathBuf::from("A/")
			})
		);

		assert_eq!(storage.length().unwrap(), 0);
	}

	#[test]
	fn simple_delete_on_existing() {
		let (
			storage,
			prefix,
			root_etag,
			A_etag,
			_,
			AB_etag,
			AAA_etag,
			public_etag,
			B_etag,
			BA_etag,
		) = build_test_db();

		let old_AAA_etag = delete(
			&storage,
			&prefix,
			&std::path::Path::new("A/AA/AAA"),
			&crate::Etag::from(""),
		)
		.unwrap();

		assert_eq!(AAA_etag, old_AAA_etag);

		/*
		assert!(tmp_folder_path.exists());
		assert!(tmp_folder_path.join(".folder.itemdata.toml").exists());
		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_ne!(root_datafile.etag, root_etag);

		assert!(tmp_folder_path.join("A").exists());
		assert!(tmp_folder_path
			.join("A")
			.join(".folder.itemdata.toml")
			.exists());
		let A_datafile: crate::DataFolder = toml::from_slice(
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
		let AB_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(AB_datafile.etag, AB_etag);

		assert!(tmp_folder_path.join("public").exists());
		assert!(tmp_folder_path
			.join("public")
			.join(".folder.itemdata.toml")
			.exists());
		let public_datafile: crate::DataFolder = toml::from_slice(
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
		let B_datafile: crate::DataFolder = toml::from_slice(
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
		let BA_datafile: crate::DataDocument = toml::from_slice(
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
		*/

		todo!()
	}

	#[test]
	fn does_not_works_for_folders() {
		let (
			storage,
			prefix,
			root_etag,
			A_etag,
			AA_etag,
			AB_etag,
			AAA_etag,
			public_etag,
			B_etag,
			BA_etag,
		) = build_test_db();

		assert_eq!(
			*delete(
				&storage,
				&prefix,
				&std::path::Path::new("A/AA/"),
				&crate::Etag::from(""),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::DoesNotWorksForFolders,
		);

		/*
		assert!(tmp_folder_path.exists());
		assert!(tmp_folder_path.join(".folder.itemdata.toml").exists());
		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(root_datafile.etag, root_etag);

		assert!(tmp_folder_path.join("A").exists());
		assert!(tmp_folder_path
			.join("A")
			.join(".folder.itemdata.toml")
			.exists());
		let A_datafile: crate::DataFolder = toml::from_slice(
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
		let AA_datafile: crate::DataFolder = toml::from_slice(
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
		let AAA_datafile: crate::DataDocument = toml::from_slice(
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
		let AB_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(AB_datafile.etag, AB_etag);

		assert!(tmp_folder_path.join("public").exists());
		assert!(tmp_folder_path
			.join("public")
			.join(".folder.itemdata.toml")
			.exists());
		let public_datafile: crate::DataFolder = toml::from_slice(
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
		let B_datafile: crate::DataFolder = toml::from_slice(
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
		let BA_datafile: crate::DataDocument = toml::from_slice(
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
		*/

		todo!()
	}

	#[test]
	fn delete_with_if_match_not_found() {
		let (
			storage,
			prefix,
			root_etag,
			A_etag,
			AA_etag,
			AB_etag,
			AAA_etag,
			public_etag,
			B_etag,
			BA_etag,
		) = build_test_db();

		assert_eq!(
			*delete(
				&storage,
				&prefix,
				&std::path::Path::new("A/AA/AAA"),
				&crate::Etag::from("OTHER_ETAG"),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::GetError(GetError::NoIfMatch {
				item_path: std::path::PathBuf::from("A/AA/AAA"),
				found: AAA_etag.clone(),
				search: crate::Etag::from("OTHER_ETAG")
			})
		);

		/*
		assert!(tmp_folder_path.exists());
		assert!(tmp_folder_path.join(".folder.itemdata.toml").exists());
		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(root_datafile.etag, root_etag);

		assert!(tmp_folder_path.join("A").exists());
		assert!(tmp_folder_path
			.join("A")
			.join(".folder.itemdata.toml")
			.exists());
		let A_datafile: crate::DataFolder = toml::from_slice(
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
		let AA_datafile: crate::DataFolder = toml::from_slice(
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
		let AAA_datafile: crate::DataDocument = toml::from_slice(
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
		let AB_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(AB_datafile.etag, AB_etag);

		assert!(tmp_folder_path.join("public").exists());
		assert!(tmp_folder_path
			.join("public")
			.join(".folder.itemdata.toml")
			.exists());
		let public_datafile: crate::DataFolder = toml::from_slice(
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
		let B_datafile: crate::DataFolder = toml::from_slice(
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
		let BA_datafile: crate::DataDocument = toml::from_slice(
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
		*/

		todo!()
	}

	#[test]
	fn delete_with_if_match_found() {
		let (
			storage,
			prefix,
			root_etag,
			A_etag,
			_,
			AB_etag,
			AAA_etag,
			public_etag,
			B_etag,
			BA_etag,
		) = build_test_db();

		let old_AAA_etag = delete(
			&storage,
			&prefix,
			&std::path::Path::new("A/AA/AAA"),
			&AAA_etag,
		)
		.unwrap();

		assert_eq!(old_AAA_etag, AAA_etag);

		/*
		assert!(tmp_folder_path.exists());
		assert!(tmp_folder_path.join(".folder.itemdata.toml").exists());
		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_ne!(root_datafile.etag, root_etag);

		assert!(tmp_folder_path.join("A").exists());
		assert!(tmp_folder_path
			.join("A")
			.join(".folder.itemdata.toml")
			.exists());
		let A_datafile: crate::DataFolder = toml::from_slice(
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
		let AB_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(AB_datafile.etag, AB_etag);

		assert!(tmp_folder_path.join("public").exists());
		assert!(tmp_folder_path
			.join("public")
			.join(".folder.itemdata.toml")
			.exists());
		let public_datafile: crate::DataFolder = toml::from_slice(
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
		let B_datafile: crate::DataFolder = toml::from_slice(
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
		let BA_datafile: crate::DataDocument = toml::from_slice(
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
		*/

		todo!()
	}

	#[test]
	fn delete_with_if_match_all() {
		let (
			storage,
			prefix,
			root_etag,
			A_etag,
			_,
			AB_etag,
			AAA_etag,
			public_etag,
			B_etag,
			BA_etag,
		) = build_test_db();

		let old_AAA_etag = delete(
			&storage,
			&prefix,
			&std::path::Path::new("A/AA/AAA"),
			&crate::Etag::from("*"),
		)
		.unwrap();

		assert_eq!(old_AAA_etag, AAA_etag);

		/*
		assert!(tmp_folder_path.exists());
		assert!(tmp_folder_path.join(".folder.itemdata.toml").exists());
		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_ne!(root_datafile.etag, root_etag);

		assert!(tmp_folder_path.join("A").exists());
		assert!(tmp_folder_path
			.join("A")
			.join(".folder.itemdata.toml")
			.exists());
		let A_datafile: crate::DataFolder = toml::from_slice(
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
		let AB_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(AB_datafile.etag, AB_etag);

		assert!(tmp_folder_path.join("public").exists());
		assert!(tmp_folder_path
			.join("public")
			.join(".folder.itemdata.toml")
			.exists());
		let public_datafile: crate::DataFolder = toml::from_slice(
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
		let B_datafile: crate::DataFolder = toml::from_slice(
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
		let BA_datafile: crate::DataDocument = toml::from_slice(
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
		*/

		todo!()
	}

	#[test]
	fn delete_with_existing_folder_conflict() {
		let (
			storage,
			prefix,
			root_etag,
			A_etag,
			AA_etag,
			AB_etag,
			AAA_etag,
			public_etag,
			B_etag,
			BA_etag,
		) = build_test_db();

		assert_eq!(
			*delete(
				&storage,
				&prefix,
				&std::path::Path::new("A/AA"),
				&crate::Etag::from(""),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::GetError(GetError::Conflict {
				item_path: std::path::PathBuf::from("A/AA/")
			})
		);

		/*
		assert!(tmp_folder_path.exists());
		assert!(tmp_folder_path.join(".folder.itemdata.toml").exists());
		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(root_datafile.etag, root_etag);

		assert!(tmp_folder_path.join("A").exists());
		assert!(tmp_folder_path
			.join("A")
			.join(".folder.itemdata.toml")
			.exists());
		let A_datafile: crate::DataFolder = toml::from_slice(
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
		let AA_datafile: crate::DataFolder = toml::from_slice(
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
		let AAA_datafile: crate::DataDocument = toml::from_slice(
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
		let AB_datafile: crate::DataDocument = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join("A").join(".AB.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_eq!(AB_datafile.etag, AB_etag);

		assert!(tmp_folder_path.join("public").exists());
		assert!(tmp_folder_path
			.join("public")
			.join(".folder.itemdata.toml")
			.exists());
		let public_datafile: crate::DataFolder = toml::from_slice(
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
		let B_datafile: crate::DataFolder = toml::from_slice(
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
		let BA_datafile: crate::DataDocument = toml::from_slice(
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
		*/

		todo!()
	}

	#[test]
	fn delete_in_public() {
		let (storage, prefix, root_etag, A_etag, AA_etag, AB_etag, AAA_etag, _, _, _) =
			build_test_db();

		delete(
			&storage,
			&prefix,
			&std::path::Path::new("public/B/BA"),
			&crate::Etag::from(""),
		)
		.unwrap();

		/*
		assert!(tmp_folder_path.exists());
		assert!(tmp_folder_path.join(".folder.itemdata.toml").exists());
		let root_datafile: crate::DataFolder = toml::from_slice(
			&std::fs::read(&tmp_folder_path.join(".folder.itemdata.toml")).unwrap(),
		)
		.unwrap();
		assert_ne!(root_datafile.etag, root_etag);

		assert!(tmp_folder_path.join("A").exists());
		assert!(tmp_folder_path
			.join("A")
			.join(".folder.itemdata.toml")
			.exists());
		let A_datafile: crate::DataFolder = toml::from_slice(
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
		let AA_datafile: crate::DataFolder = toml::from_slice(
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
		let AAA_datafile: crate::DataDocument = toml::from_slice(
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
		let AB_datafile: crate::DataDocument = toml::from_slice(
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
		*/

		todo!()
	}

	#[test]
	fn delete_in_incorrect_path() {
		let storage = LocalStorageMock::new();
		let prefix = String::from("pontus_onyx_delete_test");

		assert_eq!(
			*delete(
				&storage,
				&prefix,
				&std::path::Path::new("A/../AA"),
				&crate::Etag::from(""),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::GetError(GetError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/../"),
				error: String::from("`..` is not allowed")
			})
		);

		assert_eq!(storage.length().unwrap(), 0);
	}
}
*/
