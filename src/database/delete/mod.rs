impl super::Database {
	pub fn delete(
		&mut self,
		path: &std::path::PathBuf,
		if_match: &crate::Etag,
	) -> Result<crate::Etag, ErrorDelete> {
		/*
		TODO : option to keep old documents ?
			A provider MAY offer version rollback functionality to its users,
			but this specification does not define the interface for that.
		*/

		self.source
			.delete(path, if_match)
			.map(|item| Ok(item))
			.unwrap_or(Err(ErrorDelete::InternalError)) // TODO
	}
}

mod error;
pub use error::*;
