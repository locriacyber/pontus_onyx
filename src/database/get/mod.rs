impl super::Database {
	pub fn get(
		&self,
		path: &std::path::PathBuf,
		if_match: &crate::Etag,
		if_none_match: &[&crate::Etag],
	) -> Result<crate::Item, ErrorGet> {
		self.source
			.read(path, if_match, if_none_match, true)
			.map(|item| Ok(item))
			.unwrap_or(Err(ErrorGet::InternalError)) // TODO
	}
}

mod error;
pub use error::*;
