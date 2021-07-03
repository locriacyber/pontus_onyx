pub fn delete(
	_root_folder_path: &std::path::PathBuf,
	_path: &std::path::PathBuf,
	_if_match: &crate::Etag,
) -> Result<crate::Etag, DeleteError> {
	todo!()
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeleteError {}
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
// TODO : public_display (without details)
// TODO : to_http_response
impl std::error::Error for DeleteError {}
