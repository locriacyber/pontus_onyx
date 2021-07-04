impl super::Database {
	pub fn get(
		&self,
		path: &std::path::Path,
		if_match: &crate::Etag,
		if_none_match: &[&crate::Etag],
	) -> Result<crate::Item, Box<dyn std::any::Any>> {
		self.source.read(path, if_match, if_none_match, true)
	}
}
