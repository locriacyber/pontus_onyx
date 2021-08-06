/// It's the folder name, between an `/` and another `/`, or the filename.
///
/// See also [`ItemPath`][`crate::item::ItemPath`].
#[derive(PartialEq, Eq, Clone)]
pub enum ItemPathPart {
	Folder(String),
	Document(String),
}
impl ItemPathPart {
	pub fn name(&self) -> &str {
		match self {
			Self::Folder(name) => name,
			Self::Document(name) => name,
		}
	}
	/// Check if a name of the part of a path does not contains unauthorized content.
	pub fn check_validity(&self, check_itemdata: bool) -> Result<(), String> {
		if self.name().trim().is_empty() {
			return Err(String::from("should not be empty"));
		}

		if self.name().trim() == "." {
			return Err(String::from("`.` is not allowed"));
		}

		if self.name().trim() == ".." {
			return Err(String::from("`..` is not allowed"));
		}

		if self.name().trim() == "folder" {
			return Err(String::from("`folder` is not allowed"));
		}

		if self.name().contains('/') {
			return Err(format!(
				"`{}` should not contains `/` character",
				self.name()
			));
		}

		if self.name().contains('\\') {
			return Err(format!(
				"`{}` should not contains `\\` character",
				self.name()
			));
		}

		if self.name().contains('\0') {
			return Err(format!(
				"`{}` should not contains `\\0` character",
				self.name()
			));
		}

		if check_itemdata && self.name().contains(".itemdata.") {
			return Err(format!(
				"`{}` should not contains `.itemdata.` string",
				self.name()
			));
		}

		return Ok(());
	}
}
impl std::fmt::Display for ItemPathPart {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Folder(name) => f.write_fmt(format_args!("{}/", name)),
			Self::Document(name) => f.write_str(name),
		}
	}
}

impl std::fmt::Debug for ItemPathPart {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Folder(name) => f.write_fmt(format_args!("Folder({:?})", name)),
			Self::Document(name) => f.write_fmt(format_args!("Document({:?})", name)),
		}
	}
}
