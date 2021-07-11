pub fn delete(
	_root_folder_path: &std::path::Path,
	_path: &std::path::Path,
	_if_match: &crate::Etag,
) -> Result<crate::Etag, Box<dyn std::error::Error>> {
	Err(Box::new(DeleteError::InternalError))
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeleteError {
	InternalError,
}
impl std::fmt::Display for DeleteError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		/*
		match self {
			Self::Conflict{item_path} => f.write_fmt(format_args!("name conflict between folder and file on the path `{}`", item_path)),
			Self::NotFound{item_path} => f.write_fmt(format_args!("path not found : `{}`", item_path)),
			Self::NoContentInside{item_path} => f.write_fmt(format_args!("no content found in `{}`", item_path)),
		}
		*/
		f.write_str("TODO")
	}
}
impl std::error::Error for DeleteError {}
impl crate::database::Error for DeleteError {
	fn to_response(&self, _: &str, _: bool) -> actix_web::HttpResponse {
		todo!() // TODO
	}
}
