impl super::Database {
	pub fn put(
		&mut self,
		path: &std::path::Path,
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

		self.source.put(path, if_match, if_none_match, content)
	}
}

#[derive(Debug)]
pub enum ResultPut {
	Created(crate::Etag),
	Updated(crate::Etag),
	Err(Box<dyn std::any::Any>),
}
impl ResultPut {
	pub fn unwrap(self) -> crate::Etag {
		match self {
			Self::Created(etag) => etag,
			Self::Updated(etag) => etag,
			Self::Err(_) => panic!(),
		}
	}
	pub fn unwrap_err(self) -> Box<dyn std::any::Any> {
		match self {
			Self::Created(_) => panic!(),
			Self::Updated(_) => panic!(),
			Self::Err(e) => e,
		}
	}
}
