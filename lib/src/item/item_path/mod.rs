mod item_path_part;

#[cfg(test)]
mod tests;

pub use item_path_part::ItemPathPart;

/// Used to describe path of an [`Item`][`crate::item::Item`] in database.
///
/// It is a vector of [`ItemPathPart`][`crate::item::ItemPathPart`].
#[derive(PartialEq, Eq, Clone)]
pub struct ItemPath(Vec<ItemPathPart>);

impl ItemPath {
	pub fn joined(&self, part: &ItemPathPart) -> Result<Self, String> {
		if self.is_document() {
			return Err(String::from("last item is a document"));
		} else {
			let mut parts = self.0.clone();

			if part.name().is_empty() {
				if let Some(ItemPathPart::Folder(last_name)) = self.0.last() {
					if !last_name.is_empty() {
						parts.push(part.clone());
					}
				}
			} else {
				if let Some(item) = self.0.last() {
					if item.name().is_empty() {
						parts.pop().unwrap();
					}
				}

				parts.push(part.clone());
			}

			return Ok(Self(parts));
		}
	}
	pub fn joined_folder(&self, name: &str) -> Result<Self, String> {
		self.joined(&ItemPathPart::Folder(String::from(name)))
	}
	pub fn joined_doc(&self, name: &str) -> Result<Self, String> {
		self.joined(&ItemPathPart::Document(String::from(name)))
	}
	pub fn folder_clone(&self) -> Self {
		let mut parts = self.0.clone();
		*parts.last_mut().unwrap() =
			ItemPathPart::Folder(String::from(parts.last().unwrap().name()));

		Self(parts)
	}
	pub fn document_clone(&self) -> Self {
		let mut parts = self.0.clone();
		*parts.last_mut().unwrap() =
			ItemPathPart::Document(String::from(parts.last().unwrap().name()));

		Self(parts)
	}
	pub fn file_name(&self) -> &str {
		match self.0.last() {
			Some(item) => item.name(),
			None => "",
		}
	}
	pub fn parent(&self) -> Option<Self> {
		let mut result = self.0.clone();

		result.pop()?;

		if !result.is_empty() {
			return Some(Self(result));
		} else if self.file_name().is_empty() {
			return None;
		} else {
			return Some(Self(vec![ItemPathPart::Folder(String::new())]));
		}
	}
	pub fn starts_with(&self, other: &str) -> bool {
		format!("{}", self).starts_with(other)
	}
	pub fn ends_with(&self, other: &str) -> bool {
		format!("{}", self).ends_with(other)
	}
	pub fn is_folder(&self) -> bool {
		matches!(self.0.last(), Some(ItemPathPart::Folder(_)))
	}
	pub fn is_document(&self) -> bool {
		matches!(self.0.last(), Some(ItemPathPart::Document(_)))
	}
	pub fn parts_iter(&self) -> ItemPathPartIterator {
		ItemPathPartIterator {
			parts: self,
			current_pos: 0,
		}
	}
	// TODO : make it Iterator ?
	pub fn ancestors(&self) -> Vec<ItemPath> {
		let mut result = vec![];

		let mut cumulated = vec![];
		for part in self.0.iter() {
			cumulated.push(part.clone());

			result.push(ItemPath(cumulated.clone()));
		}

		if result
			.first()
			.unwrap_or(&crate::item::ItemPath::from("everything_else"))
			!= &crate::item::ItemPath::from("")
		{
			result.insert(0, crate::item::ItemPath::from(""));
		}

		return result;
	}
}
// TODO : `impl AsRef<ItemPath> for &str` ?

impl From<&str> for ItemPath {
	fn from(input: &str) -> Self {
		let mut result = vec![];

		let input = input
			.trim()
			.strip_prefix('/')
			.unwrap_or(input)
			.strip_prefix('\\')
			.unwrap_or(input);

		for slash_stage in input.split('/') {
			for backslash_stage in slash_stage.trim().split('\\') {
				if backslash_stage.trim() == ".." {
					result.pop();
					if result.first().is_none() {
						result.push(ItemPathPart::Folder(String::new()));
					}
				} else if backslash_stage.trim() == "." {
					// nothing to add

					if result.first().is_none() {
						result.push(ItemPathPart::Folder(String::new()));
					}
				} else {
					if let Some(ItemPathPart::Folder(folder_name)) = result.first() {
						if folder_name.is_empty() {
							result.remove(0);
						}
					}

					if result.len() > 1
						&& result
							.last()
							.unwrap_or(&ItemPathPart::Folder(String::from("anything_else")))
							.name()
							.is_empty()
					{
						result.pop();
					}

					result.push(ItemPathPart::Folder(String::from(backslash_stage.trim())));
				}
			}
		}

		match result.last_mut() {
			Some(last) => {
				if last.name().is_empty() {
					if result.len() > 1 {
						result.pop();
					}
				} else {
					*last = ItemPathPart::Document(String::from(last.name()));
				}
			}
			None => result.push(ItemPathPart::Folder(String::new())),
		}

		return Self(result);
	}
}

impl std::convert::From<&ItemPath> for std::path::PathBuf {
	fn from(input: &ItemPath) -> Self {
		std::path::PathBuf::from(
			&format!("{}", input).replace('/', &String::from(std::path::MAIN_SEPARATOR)),
		)
	}
}

impl From<&std::path::Path> for ItemPath {
	fn from(input: &std::path::Path) -> Self {
		ItemPath::from(&*input.to_string_lossy())
	}
}

impl std::fmt::Display for ItemPath {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		for (i, part) in self.0.iter().enumerate() {
			if !(i == 0 && part.name().is_empty()) {
				if let Err(error) = f.write_fmt(format_args!("{}", part)) {
					return Err(error);
				}
			}
		}

		Ok(())
	}
}

impl std::fmt::Debug for ItemPath {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		f.write_fmt(format_args!(
			"[{}]",
			self.0.iter().fold(String::new(), |mut acc, e| {
				if !acc.is_empty() {
					acc += ", ";
				}
				acc += &format!("{:?}", e);

				acc
			})
		))
	}
}

/// An iterator of [`ItemPathPart`].
pub struct ItemPathPartIterator<'a> {
	parts: &'a ItemPath,
	current_pos: usize,
}
impl<'a> Iterator for ItemPathPartIterator<'a> {
	type Item = &'a ItemPathPart;
	fn next(&mut self) -> Option<Self::Item> {
		let result = self.parts.0.get(self.current_pos);
		self.current_pos += 1;

		return result;
	}
}
