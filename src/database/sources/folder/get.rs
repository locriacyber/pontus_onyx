pub fn get(
	_root_folder_path: &std::path::Path,
	_path: &std::path::Path,
	_recursive: bool,
) -> Result<crate::Item, Box<dyn std::any::Any>> {


	todo!()
}

#[derive(Debug, PartialEq, Eq)]
pub enum GetError {
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
}
impl std::fmt::Display for GetError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		f.write_str("TODO")
	}
}
impl std::error::Error for GetError {}
impl crate::database::Error for GetError {
	fn to_response(&self, _: &str, _: bool) -> actix_web::HttpResponse {
		todo!() // TODO
	}
}
