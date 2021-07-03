impl super::Database {
	pub fn put(
		&mut self,
		path: &std::path::PathBuf,
		content: crate::Item,
		if_match: &crate::Etag,
		if_none_match: &[&crate::Etag],
	) -> ResultPut {
		/*
		TODO :
			* its version being updated, as well as that of its parent folder
				and further ancestor folders, using a strong validator [HTTP,
				section 7.2].
		*/

		self.source
			.put(path, if_match, if_none_match, content)
			.map(|item| ResultPut::Created(item))
			.unwrap_or(ResultPut::Err(ErrorPut::InternalError)) // TODO
	}
}

pub enum ResultPut {
	Created(crate::Etag),
	Updated(crate::Etag),
	Err(ErrorPut),
}

mod error;
pub use error::*;
