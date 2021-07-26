#![allow(clippy::needless_return)]
#![allow(non_snake_case)]

#[cfg(feature = "client_lib")]
pub mod client;

#[cfg(feature = "server_lib")]
pub mod database;

/// Content-Type is a String value, used to indicate what kind of data
/// is (binary) content of an [`Document`][`crate::Item::Document`].
///
/// [More on MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
#[serde(from = "String", into = "String")]
pub struct ContentType(String);
impl From<String> for ContentType {
	fn from(input: String) -> Self {
		Self(input)
	}
}
impl From<&str> for ContentType {
	fn from(input: &str) -> Self {
		Self(String::from(input))
	}
}
impl From<ContentType> for String {
	fn from(input: ContentType) -> Self {
		input.0
	}
}
impl std::cmp::PartialEq<&str> for ContentType {
	fn eq(&self, other: &&str) -> bool {
		self.0 == *other
	}
}
impl std::cmp::PartialEq<&str> for &ContentType {
	fn eq(&self, other: &&str) -> bool {
		self.0 == *other
	}
}
impl std::fmt::Display for ContentType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		self.0.fmt(f)
	}
}

/// ETag is String value, used for versioning purposes.
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
pub struct Etag(String);
impl From<String> for Etag {
	fn from(input: String) -> Self {
		Self(input)
	}
}
impl From<&str> for Etag {
	fn from(input: &str) -> Self {
		Self(String::from(input))
	}
}
impl From<Etag> for String {
	fn from(input: Etag) -> Self {
		input.0
	}
}
impl std::cmp::PartialEq<&str> for Etag {
	fn eq(&self, other: &&str) -> bool {
		self.0 == *other
	}
}
impl std::cmp::PartialEq<&str> for &Etag {
	fn eq(&self, other: &&str) -> bool {
		self.0 == *other
	}
}
impl std::fmt::Debug for Etag {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		f.write_fmt(format_args!("Etag(\"{}\")", self.0))
	}
}
impl std::fmt::Display for Etag {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		self.0.fmt(f)
	}
}
impl Etag {
	#![allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self(ulid::Ulid::new().to_string())
	}
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
	pub fn trim(&self) -> Self {
		Self(self.0.trim().into())
	}
	pub fn to_uppercase(&self) -> Self {
		Self(self.0.to_uppercase())
	}
}

/// It represent all data of an endpoint of a path in database.
///
/// It contains the requested content, but also its metadata, like [`Etag`], for example.
///
/// Typically, Item should be returned by database when GET a path.
#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[derivative(Debug)]
pub enum Item {
	Folder {
		etag: crate::Etag,
		#[derivative(Debug = "ignore")]
		/// They are other items inside this folder, [`Folder`][`crate::Item::Folder`] or [`Document`][`crate::Item::Document`].
		///
		/// Its (String) keys are their names.
		/// Like `my_folder` for a [`Folder`][`crate::Item::Folder`],
		/// or `example.json` for a [`Document`][`crate::Item::Document`].
		///
		/// It can be [`None`][`Option::None`] if we don't need to fetch children, for performances purposes.
		content: Option<std::collections::HashMap<String, Box<crate::Item>>>,
	},
	Document {
		etag: crate::Etag,
		#[derivative(Debug = "ignore")]
		/// The binary content of this document.
		///
		/// It can be [`None`][`Option::None`] if we don't need to fetch its content, for performances purposes.
		content: Option<Vec<u8>>,
		content_type: crate::ContentType,
		last_modified: chrono::DateTime<chrono::offset::Utc>,
	},
}
impl Item {
	/// Creates a new [`Folder`][`crate::Item::Folder`], easier.
	pub fn new_folder(easy_content: Vec<(&str, Self)>) -> Self {
		let mut content = std::collections::HashMap::new();
		for (name, item) in easy_content {
			content.insert(String::from(name), Box::new(item));
		}

		return Self::Folder {
			etag: crate::Etag::new(),
			content: Some(content),
		};
	}

	/// Creates a new [`Document`][`crate::Item::Document`], easier.
	pub fn new_doc(content: &[u8], content_type: &str) -> Self {
		return Self::Document {
			etag: crate::Etag::new(),
			content: Some(content.to_vec()),
			content_type: crate::ContentType::from(content_type),
			last_modified: chrono::Utc::now(),
		};
	}

	/// Create a clone of this [`Item`][`crate::Item`], but with its `content` as `None`.
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

	/// Check if this is the [`Folder`][`crate::Item::Folder`] variant.
	pub fn is_folder(&self) -> bool {
		match self {
			Self::Folder { .. } => true,
			Self::Document { .. } => false,
		}
	}

	/// Check if this is the [`Document`][`crate::Item::Document`] variant.
	pub fn is_document(&self) -> bool {
		match self {
			Self::Document { .. } => true,
			Self::Folder { .. } => false,
		}
	}

	/// Get the [`Etag`] of this [`Item`][`crate::Item`].
	pub fn get_etag(&self) -> &crate::Etag {
		return match self {
			Self::Folder { etag, .. } => etag,
			Self::Document { etag, .. } => etag,
		};
	}

	/// If this is an [`Document`][`crate::Item::Document`] and with
	/// an [`Some`][`Option::Some`] as `content`, should returns the
	/// binary content. Otherwise, returns [`None`][`Option::None`].
	pub fn get_document_content(self) -> Option<Vec<u8>> {
		match self {
			Self::Document { content, .. } => content,
			Self::Folder { .. } => None,
		}
	}

	/// If this is an [`Folder`][`crate::Item::Folder`], it should returns the child `path` from its `content`.
	pub fn get_child_mut(&mut self, path: &std::path::Path) -> Option<&mut Self> {
		let parents = {
			let ancestors = path.ancestors();
			let mut paths: Vec<&std::path::Path> = ancestors.into_iter().collect();
			paths = paths
				.into_iter()
				.rev()
				.skip(1)
				.take(ancestors.count().saturating_sub(1))
				.collect();
			paths
		};

		let mut pending_parent = Some(Box::new(self));
		for path_part in parents {
			if let Some(boxed_parent) = pending_parent {
				match *boxed_parent {
					Self::Folder {
						content: Some(content),
						..
					} => {
						pending_parent = content
							.get_mut(path_part.file_name().unwrap().to_str().unwrap())
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

	/// If this is an [`Folder`][`crate::Item::Folder`], it should returns the child `path` from its `content`.
	pub fn get_child(&self, path: &std::path::Path) -> Option<&Self> {
		let parents = {
			let ancestors = path.ancestors();
			let mut paths: Vec<&std::path::Path> = ancestors.into_iter().collect();
			paths = paths
				.into_iter()
				.rev()
				.skip(1)
				.take(ancestors.count().saturating_sub(1))
				.collect();
			paths
		};

		let mut pending_parent = Some(Box::new(self));
		for path_part in parents {
			if let Some(boxed_parent) = pending_parent {
				match *boxed_parent {
					Self::Folder {
						content: Some(content),
						..
					} => {
						pending_parent = content
							.get(path_part.file_name().unwrap().to_str().unwrap())
							.map(|e| Box::new(&**e));
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

#[test]
fn jlupvpfbk7wbig1at4h() {
	let AAA = crate::Item::new_doc(b"test", "text/plain");
	let AA = crate::Item::new_folder(vec![("AAA.txt", AAA.clone())]);
	let A = crate::Item::new_folder(vec![("AA", AA.clone())]);
	let root = crate::Item::new_folder(vec![("A", A.clone())]);

	assert_eq!(
		root.get_child(&std::path::Path::new("A/AA/AAA.txt")),
		Some(&AAA)
	);
	assert_eq!(root.get_child(&std::path::Path::new("A/AA/")), Some(&AA));
	assert_eq!(root.get_child(&std::path::Path::new("A/AA")), Some(&AA));
	assert_eq!(root.get_child(&std::path::Path::new("A/")), Some(&A));
	assert_eq!(root.get_child(&std::path::Path::new("A")), Some(&A));
	assert_eq!(root.get_child(&std::path::Path::new("")), Some(&root));

	assert_eq!(root.get_child(&std::path::Path::new("B")), None);
	assert_eq!(root.get_child(&std::path::Path::new("B/")), None);
	assert_eq!(root.get_child(&std::path::Path::new("B/BB")), None);
	assert_eq!(root.get_child(&std::path::Path::new("B/BB/")), None);
}

#[test]
fn j5bmhdxhlgkhdk82rjio3ej6() {
	let mut AAA = crate::Item::new_doc(b"test", "text/plain");
	let mut AA = crate::Item::new_folder(vec![("AAA.txt", AAA.clone())]);
	let mut A = crate::Item::new_folder(vec![("AA", AA.clone())]);
	let mut root = crate::Item::new_folder(vec![("A", A.clone())]);

	assert_eq!(
		root.get_child_mut(&std::path::Path::new("A/AA/AAA.txt")),
		Some(&mut AAA)
	);
	assert_eq!(
		root.get_child_mut(&std::path::Path::new("A/AA/")),
		Some(&mut AA)
	);
	assert_eq!(
		root.get_child_mut(&std::path::Path::new("A/AA")),
		Some(&mut AA)
	);
	assert_eq!(
		root.get_child_mut(&std::path::Path::new("A/")),
		Some(&mut A)
	);
	assert_eq!(root.get_child_mut(&std::path::Path::new("A")), Some(&mut A));
	assert!(root.get_child_mut(&std::path::Path::new("")).is_some());

	assert_eq!(root.get_child_mut(&std::path::Path::new("B")), None);
	assert_eq!(root.get_child_mut(&std::path::Path::new("B/")), None);
	assert_eq!(root.get_child_mut(&std::path::Path::new("B/BB")), None);
	assert_eq!(root.get_child_mut(&std::path::Path::new("B/BB/")), None);
}

pub fn item_name_is_ok(path: &str) -> Result<(), String> {
	if path.trim().is_empty() {
		return Err(String::from("should not be empty"));
	}

	if path.trim() == "." {
		return Err(String::from("`.` is not allowed"));
	}

	if path.trim() == ".." {
		return Err(String::from("`..` is not allowed"));
	}

	if path.trim() == "folder" {
		return Err(String::from("`folder` is not allowed"));
	}

	if path.contains('\0') {
		return Err(format!("`{}` should not contains \\0 character", path));
	}

	return Ok(());
}

#[test]
fn pfuh8x4mntyi3ej() {
	let input = "gq7tib";
	assert_eq!(item_name_is_ok(&input), Ok(()));
}

#[test]
fn b2auwz1qizhfkrolm() {
	let input = "";
	assert_eq!(
		item_name_is_ok(&input),
		Err(String::from("should not be empty"))
	);
}

#[test]
fn hf1atgq7tibjv22p2whyhrl() {
	let input = "gq7t\0ib";
	assert_eq!(
		item_name_is_ok(&input),
		Err(format!("`{}` should not contains \\0 character", input))
	);
}

/// Used to (de)serialize metadata of [`Document`][`crate::Item::Document`] in/from a file.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct DataDocument {
	pub datastruct_version: String,
	pub etag: crate::Etag,
	pub content_type: crate::ContentType,
	pub last_modified: chrono::DateTime<chrono::Utc>,
}
impl Default for DataDocument {
	fn default() -> Self {
		Self {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: crate::Etag::new(),
			content_type: crate::ContentType::from("application/octet-stream"),
			last_modified: chrono::Utc::now(),
		}
	}
}
impl std::convert::TryFrom<crate::Item> for DataDocument {
	type Error = String;

	fn try_from(input: crate::Item) -> Result<Self, Self::Error> {
		match input {
			crate::Item::Document {
				etag,
				content_type,
				last_modified,
				..
			} => Ok(Self {
				datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
				etag,
				content_type,
				last_modified,
			}),
			_ => Err(String::from("input should be Item::Document and it is not")),
		}
	}
}

/// Used to (de)serialize metadata of [`Folder`][`crate::Item::Folder`] in/from a file.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct DataFolder {
	pub datastruct_version: String,
	pub etag: crate::Etag,
}
impl Default for DataFolder {
	fn default() -> Self {
		Self {
			datastruct_version: String::from(env!("CARGO_PKG_VERSION")),
			etag: crate::Etag::new(),
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum ScopeRightType {
	Read,
	ReadWrite,
}
impl std::convert::TryFrom<&str> for ScopeRightType {
	type Error = ScopeParsingError;

	fn try_from(input: &str) -> Result<Self, Self::Error> {
		let input = input.trim();

		if input == "rw" {
			Ok(Self::ReadWrite)
		} else if input == "r" {
			Ok(Self::Read)
		} else {
			Err(ScopeParsingError::IncorrectRight(String::from(input)))
		}
	}
}
impl std::fmt::Display for ScopeRightType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		f.write_str(match &self {
			Self::Read => "read only",
			Self::ReadWrite => "read and write",
		})
	}
}

#[derive(Debug, PartialEq)]
pub enum ScopeParsingError {
	IncorrectFormat(String),
	IncorrectModule(String),
	IncorrectRight(String),
}
