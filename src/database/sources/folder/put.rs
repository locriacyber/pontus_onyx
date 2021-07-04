pub fn put(
	_root_folder_path: &std::path::Path,
	_path: &std::path::Path,
	_if_match: &crate::Etag,
	_if_none_match: &[&crate::Etag],
	_item: crate::Item,
) -> crate::database::put::ResultPut {
	crate::database::put::ResultPut::Err(Box::new(PutError::InternalError))
}

#[derive(Debug, PartialEq, Eq)]
pub enum PutError {
	InternalError,
}
impl std::fmt::Display for PutError {
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
impl std::error::Error for PutError {}
impl crate::database::Error for PutError {
	fn to_response(&self, _: &str, _: bool) -> actix_web::HttpResponse {
		todo!() // TODO
	}
}
