mod content_type;
mod datafiles;
mod etag;
mod item_path;

pub use content_type::ContentType;
pub use datafiles::{DataDocument, DataFolder};
pub use etag::Etag;
pub use item_path::*;

#[cfg(test)]
mod tests;

/// It represent all data of an endpoint of a path in database.
///
/// It contains the requested content, but also its metadata, like [`Etag`][`crate::item::Etag`], for example.
///
/// Typically, Item should be returned by database when GET a path.
#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[derivative(Debug)]
pub enum Item {
	Folder {
		etag: crate::item::Etag,
		#[derivative(Debug = "ignore")]
		/// They are other items inside this folder, [`Folder`][`crate::item::Item::Folder`] or [`Document`][`crate::item::Item::Document`].
		///
		/// Its (String) keys are their names.
		/// Like `my_folder` for a [`Folder`][`crate::item::Item::Folder`],
		/// or `example.json` for a [`Document`][`crate::item::Item::Document`].
		///
		/// It can be [`None`][`Option::None`] if we don't need to fetch children, for performances purposes.
		content: Option<std::collections::HashMap<String, Box<crate::item::Item>>>,
	},
	Document {
		etag: crate::item::Etag,
		#[derivative(Debug = "ignore")]
		/// The binary content of this document.
		///
		/// It can be [`None`][`Option::None`] if we don't need to fetch its content, for performances purposes.
		content: Option<Vec<u8>>,
		content_type: crate::item::ContentType,
		last_modified: Option<chrono::DateTime<chrono::offset::Utc>>,
	},
}
impl Item {
	/// Creates a new [`Folder`][`crate::item::Item::Folder`], easier.
	pub fn new_folder(easy_content: Vec<(&str, Self)>) -> Self {
		let mut content = std::collections::HashMap::new();
		for (name, item) in easy_content {
			content.insert(String::from(name), Box::new(item));
		}

		return Self::Folder {
			etag: crate::item::Etag::new(),
			content: Some(content),
		};
	}

	/// Creates a new [`Document`][`crate::item::Item::Document`], easier.
	pub fn new_doc(content: &[u8], content_type: &str) -> Self {
		return Self::Document {
			etag: crate::item::Etag::new(),
			content: Some(content.to_vec()),
			content_type: crate::item::ContentType::from(content_type),
			last_modified: Some(chrono::Utc::now()),
		};
	}

	/// Create a clone of this [`Item`][`crate::item::Item`], but with its `content` as `None`.
	pub fn empty_clone(&self) -> Self {
		return match self {
			Self::Folder { etag, .. } => Self::Folder {
				etag: etag.clone(),
				content: None,
			},
			Self::Document {
				etag,
				content_type,
				last_modified,
				..
			} => Self::Document {
				etag: etag.clone(),
				content_type: content_type.clone(),
				last_modified: *last_modified,
				content: None,
			},
		};
	}

	/// Check if this is the [`Folder`][`crate::item::Item::Folder`] variant.
	pub fn is_folder(&self) -> bool {
		match self {
			Self::Folder { .. } => true,
			Self::Document { .. } => false,
		}
	}

	/// Check if this is the [`Document`][`crate::item::Item::Document`] variant.
	pub fn is_document(&self) -> bool {
		match self {
			Self::Document { .. } => true,
			Self::Folder { .. } => false,
		}
	}

	/// Get the [`Etag`][`crate::item::Etag`] of this [`Item`][`crate::item::Item`].
	pub fn get_etag(&self) -> &crate::item::Etag {
		return match self {
			Self::Folder { etag, .. } => etag,
			Self::Document { etag, .. } => etag,
		};
	}

	/// If this is an [`Document`][`crate::item::Item::Document`] and with
	/// an [`Some`][`Option::Some`] as `content`, should returns the
	/// binary content. Otherwise, returns [`None`][`Option::None`].
	pub fn get_document_content(self) -> Option<Vec<u8>> {
		match self {
			Self::Document { content, .. } => content,
			Self::Folder { .. } => None,
		}
	}

	/// If `self` is an [`Folder`][`crate::item::Item::Folder`], it should returns the child `path` from its `content`.
	pub fn get_child_mut(&mut self, path: &crate::item::ItemPath) -> Option<&mut Self> {
		let parents: Vec<crate::item::ItemPath> = path.ancestors();

		if path == &crate::item::ItemPath::from("") {
			return Some(self);
		}

		let mut pending_parent = Some(Box::new(self));
		for path_part in parents.into_iter().skip(1) {
			if let Some(boxed_parent) = pending_parent {
				match *boxed_parent {
					Self::Folder {
						content: Some(content),
						..
					} => {
						pending_parent = content
							.get_mut(path_part.file_name())
							.map(|e| Box::new(&mut **e));
					}
					_ => {
						pending_parent = None;
					}
				}
			} else {
				return None;
			}
		}

		return pending_parent.map(|e| &mut **e);
	}

	/// If `self` is an [`Folder`][`crate::item::Item::Folder`], it should returns the child `path` from its `content`.
	pub fn get_child(&self, path: &crate::item::ItemPath) -> Option<&Self> {
		let parents: Vec<crate::item::ItemPath> = path.ancestors();

		if path == &crate::item::ItemPath::from("") {
			return Some(self);
		}

		let mut pending_parent = Some(Box::new(self));
		for path_part in parents.into_iter().skip(1) {
			if let Some(boxed_parent) = pending_parent {
				match *boxed_parent {
					Self::Folder {
						content: Some(content),
						..
					} => {
						pending_parent = content.get(path_part.file_name()).map(|e| Box::new(&**e));
					}
					_ => {
						pending_parent = None;
					}
				}
			} else {
				return None;
			}
		}

		return pending_parent.map(|e| &**e);
	}
}
