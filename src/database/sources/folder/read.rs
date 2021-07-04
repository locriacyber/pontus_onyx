pub fn read(
	_root_folder_path: &std::path::Path,
	_path: &std::path::Path,
	_recursive: bool,
) -> Result<crate::Item, Box<impl crate::database::Error>> {
	Err(Box::new(ReadError::InternalError))
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReadError {
	/*
	Conflict {
		item_path: std::path::PathBuf,
	},
	NotFound {
		item_path: std::path::PathBuf,
	},
	IncorrectItemName {
		item_path: std::path::PathBuf,
		error: String,
	},
	CanNotReadFile {
		path: std::path::PathBuf,
		error: String,
	},
	CanNotDeserializeFile {
		path: std::path::PathBuf,
		error: String,
	},
	IOError {
		error: String,
	},
	NotCompatibleFileName {
		path: std::path::PathBuf,
	},
	CanNotBeListed,
	*/
	InternalError,
}
impl std::fmt::Display for ReadError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		f.write_str("TODO")
	}
}
impl std::error::Error for ReadError {}
impl crate::database::Error for ReadError {
	fn to_response(&self, _: &str, _: bool) -> actix_web::HttpResponse {
		todo!() // TODO
	}
}

/*
#[cfg(test)]
mod tests {
	#![allow(non_snake_case)]

	use super::{read, ReadError};
	use std::convert::TryFrom;

	// TODO : replace `path.push(...)` with `path.join(...)`

	#[test]
	fn f18szwvdhh23() {
		let AA = crate::Item::new_doc(b"AA", "text/plain");
		let AB = crate::Item::new_doc(b"AB", "text/plain");
		let AC = crate::Item::new_doc(b"AC", "text/plain");
		let BA = crate::Item::new_doc(b"BA", "text/plain");
		let BB = crate::Item::new_doc(b"BB", "text/plain");
		let CC = crate::Item::new_doc(b"CC", "text/plain");

		let A = crate::Item::new_folder(vec![
			("AA", AA.clone()),
			("AB", AB.clone()),
			("AC", AC.clone()),
		]);
		let B = crate::Item::new_folder(vec![("BA", BA.clone()), ("BB", BB.clone())]);
		let C = crate::Item::new_folder(vec![("CC", CC.clone())]);
		let public = crate::Item::new_folder(vec![("C", C.clone())]);

		let root = crate::Item::new_folder(vec![
			("A", A.clone()),
			("B", B.clone()),
			("public", public.clone()),
		]);

		let mut root_without_public = root.clone();
		if let crate::Item::Folder {
			content: Some(content),
			..
		} = &mut root_without_public
		{
			if let crate::Item::Folder {
				content: public_content,
				..
			} = &mut **content.get_mut("public").unwrap()
			{
				*public_content = None;
			}
		} else {
			panic!()
		}

		////////////////////////////////////////////////////////////////////////////////////////////////

		let tmp_folder = tempfile::tempdir().unwrap();
		let tmp_folder_path = tmp_folder.path().to_path_buf();

		let root_path = tmp_folder_path.clone();

		let mut A_path = tmp_folder_path.clone();
		A_path.push("A");

		let mut B_path = tmp_folder_path.clone();
		B_path.push("B");

		let mut public_path = tmp_folder_path.clone();
		public_path.push("public");

		let mut C_path = public_path.clone();
		C_path.push("C");

		let mut AA_path = A_path.clone();
		AA_path.push("AA");

		let mut AB_path = A_path.clone();
		AB_path.push("AB");

		let mut AC_path = A_path.clone();
		AC_path.push("AC");

		let mut BA_path = B_path.clone();
		BA_path.push("BA");

		let mut BB_path = B_path.clone();
		BB_path.push("BB");

		let mut CC_path = C_path.clone();
		CC_path.push("CC");

		std::fs::create_dir_all(&A_path).unwrap();
		std::fs::create_dir_all(&B_path).unwrap();
		std::fs::create_dir_all(&C_path).unwrap();

		let mut root_data_path = tmp_folder_path.clone();
		root_data_path.push(".folder.podata.toml");
		std::fs::write(
			&root_data_path,
			toml::to_string(&crate::database::DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: root.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

		let mut A_data_path = A_path.clone();
		A_data_path.push(".folder.podata.toml");
		std::fs::write(
			A_data_path,
			toml::to_string(&crate::database::DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: A.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

		let mut B_data_path = B_path.clone();
		B_data_path.push(".folder.podata.toml");
		std::fs::write(
			B_data_path,
			toml::to_string(&crate::database::DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: B.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

		let mut public_data_path = public_path.clone();
		public_data_path.push(".folder.podata.toml");
		std::fs::write(
			public_data_path,
			toml::to_string(&crate::database::DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: public.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

		let mut C_data_path = C_path.clone();
		C_data_path.push(".folder.podata.toml");
		std::fs::write(
			C_data_path,
			toml::to_string(&crate::database::DataFolder {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag: C.get_etag().clone(),
			})
			.unwrap(),
		)
		.unwrap();

		let mut AA_data_path = A_path.clone();
		AA_data_path.push(".AA.podata.toml");
		std::fs::write(
			AA_data_path,
			toml::to_string(&crate::database::DataDocument::try_from(AA.clone()).unwrap()).unwrap(),
		)
		.unwrap();

		let mut AB_data_path = A_path.clone();
		AB_data_path.push(".AB.podata.toml");
		std::fs::write(
			AB_data_path,
			toml::to_string(&crate::database::DataDocument::try_from(AB.clone()).unwrap()).unwrap(),
		)
		.unwrap();

		let mut AC_data_path = A_path.clone();
		AC_data_path.push(".AC.podata.toml");
		std::fs::write(
			AC_data_path,
			toml::to_string(&crate::database::DataDocument::try_from(AC.clone()).unwrap()).unwrap(),
		)
		.unwrap();

		let mut BA_data_path = B_path.clone();
		BA_data_path.push(".BA.podata.toml");
		std::fs::write(
			BA_data_path,
			toml::to_string(&crate::database::DataDocument::try_from(BA.clone()).unwrap()).unwrap(),
		)
		.unwrap();

		let mut BB_data_path = B_path.clone();
		BB_data_path.push(".BB.podata.toml");
		std::fs::write(
			BB_data_path,
			toml::to_string(&&crate::database::DataDocument::try_from(BB.clone()).unwrap())
				.unwrap(),
		)
		.unwrap();

		let mut CC_data_path = C_path.clone();
		CC_data_path.push(".CC.podata.toml");
		std::fs::write(
			CC_data_path,
			toml::to_string(&&crate::database::DataDocument::try_from(CC.clone()).unwrap())
				.unwrap(),
		)
		.unwrap();

		if let crate::Item::Document {
			content: Some(content),
			..
		} = &AA
		{
			std::fs::write(AA_path, content).unwrap();
		} else {
			panic!()
		}

		if let crate::Item::Document {
			content: Some(content),
			..
		} = &AB
		{
			std::fs::write(AB_path, content).unwrap();
		} else {
			panic!()
		}

		if let crate::Item::Document {
			content: Some(content),
			..
		} = &AC
		{
			std::fs::write(AC_path, content).unwrap();
		} else {
			panic!()
		}

		if let crate::Item::Document {
			content: Some(content),
			..
		} = &BA
		{
			std::fs::write(BA_path, content).unwrap();
		} else {
			panic!()
		}

		if let crate::Item::Document {
			content: Some(content),
			..
		} = &BB
		{
			std::fs::write(BB_path, content).unwrap();
		} else {
			panic!()
		}

		if let crate::Item::Document {
			content: Some(content),
			..
		} = &CC
		{
			std::fs::write(CC_path, content).unwrap();
		} else {
			panic!()
		}

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			read(&root_path, &std::path::PathBuf::from(""), true),
			Ok(root_without_public.clone())
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("A/"), true),
			Ok(A.clone())
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("A/AA"), true),
			Ok(AA.clone())
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("A/AB"), true),
			Ok(AB)
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("A/AC"), true),
			Ok(AC)
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("B/"), true),
			Ok(B)
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("B/BA"), true),
			Ok(BA)
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("B/BB"), true),
			Ok(BB)
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("public/C/CC"), true),
			Ok(CC.clone())
		);

		assert_eq!(
			read(&root_path, &std::path::PathBuf::from(""), false),
			Ok(root.empty_clone())
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("A/"), false),
			Ok(A.empty_clone())
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("A/AA"), false),
			Ok(AA.empty_clone())
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("public/"), false),
			Ok(public.empty_clone())
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("public/C/"), false),
			Ok(C.empty_clone())
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("public/C/CC"), false),
			Ok(CC.empty_clone())
		);

		////////////////////////////////////////////////////////////////////////////////////////////////

		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("A"), true),
			Err(ReadError::Conflict {
				item_path: std::path::PathBuf::from("A")
			})
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("A/AA/"), true),
			Err(ReadError::Conflict {
				item_path: std::path::PathBuf::from("A/AA/")
			})
		);
		assert_eq!(
			read(
				&root_path,
				&std::path::PathBuf::from("A/AC/not_exists"),
				true
			),
			Err(ReadError::Conflict {
				item_path: std::path::PathBuf::from("A/AC/")
			})
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("A/not_exists"), true),
			Err(ReadError::NotFound {
				item_path: std::path::PathBuf::from("A/not_exists")
			})
		);
		assert_eq!(
			read(
				&root_path,
				&std::path::PathBuf::from("A/not_exists/nested"),
				true
			),
			Err(ReadError::NotFound {
				item_path: std::path::PathBuf::from("A/not_exists/")
			})
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("B/not_exists"), true),
			Err(ReadError::NotFound {
				item_path: std::path::PathBuf::from("B/not_exists")
			})
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("not_exists/"), true),
			Err(ReadError::NotFound {
				item_path: std::path::PathBuf::from("not_exists/")
			})
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("not_exists"), true),
			Err(ReadError::NotFound {
				item_path: std::path::PathBuf::from("not_exists")
			})
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("."), true),
			Err(ReadError::IncorrectItemName {
				item_path: std::path::PathBuf::from("."),
				error: String::from("`.` is not allowed"),
			})
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("A/.."), true),
			Err(ReadError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/.."),
				error: String::from("`..` is not allowed"),
			})
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("A/../"), true),
			Err(ReadError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/../"),
				error: String::from("`..` is not allowed"),
			})
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("A/../AA"), true),
			Err(ReadError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/../"),
				error: String::from("`..` is not allowed"),
			})
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("A/A\0A"), true),
			Err(ReadError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/A\0A"),
				error: format!("`{}` should not contains \\0 character", "A\0A"),
			})
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("public/"), true),
			Err(ReadError::CanNotBeListed)
		);
		assert_eq!(
			read(&root_path, &std::path::PathBuf::from("public/C/"), true),
			Err(ReadError::CanNotBeListed)
		);

		tmp_folder.close().unwrap();
	}
}
*/
