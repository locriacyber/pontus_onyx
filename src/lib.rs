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

/// ETag is a String value, used for versioning purposes.
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

	/// If `self` is an [`Folder`][`crate::Item::Folder`], it should returns the child `path` from its `content`.
	pub fn get_child_mut(&mut self, path: &crate::ItemPath) -> Option<&mut Self> {
		let parents: Vec<crate::ItemPath> = path.ancestors();

		if path == &crate::ItemPath::from("") {
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

	/// If `self` is an [`Folder`][`crate::Item::Folder`], it should returns the child `path` from its `content`.
	pub fn get_child(&self, path: &crate::ItemPath) -> Option<&Self> {
		let parents: Vec<crate::ItemPath> = path.ancestors();

		if path == &crate::ItemPath::from("") {
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

#[test]
fn jlupvpfbk7wbig1at4h() {
	let AAA = crate::Item::new_doc(b"test", "text/plain");
	let AA = crate::Item::new_folder(vec![("AAA.txt", AAA.clone())]);
	let A = crate::Item::new_folder(vec![("AA", AA.clone())]);
	let root = crate::Item::new_folder(vec![("A", A.clone())]);

	assert_eq!(
		root.get_child(&crate::ItemPath::from("A/AA/AAA.txt")),
		Some(&AAA)
	);
	assert_eq!(root.get_child(&crate::ItemPath::from("A/AA/")), Some(&AA));
	assert_eq!(root.get_child(&crate::ItemPath::from("A/AA")), Some(&AA));
	assert_eq!(root.get_child(&crate::ItemPath::from("A/")), Some(&A));
	assert_eq!(root.get_child(&crate::ItemPath::from("A")), Some(&A));
	assert_eq!(root.get_child(&crate::ItemPath::from("")), Some(&root));

	assert_eq!(root.get_child(&crate::ItemPath::from("B")), None);
	assert_eq!(root.get_child(&crate::ItemPath::from("B/")), None);
	assert_eq!(root.get_child(&crate::ItemPath::from("B/BB")), None);
	assert_eq!(root.get_child(&crate::ItemPath::from("B/BB/")), None);
}

#[test]
fn j5bmhdxhlgkhdk82rjio3ej6() {
	let mut AAA = crate::Item::new_doc(b"test", "text/plain");
	let mut AA = crate::Item::new_folder(vec![("AAA.txt", AAA.clone())]);
	let mut A = crate::Item::new_folder(vec![("AA", AA.clone())]);
	let mut root = crate::Item::new_folder(vec![("A", A.clone())]);

	assert_eq!(
		root.get_child_mut(&crate::ItemPath::from("A/AA/AAA.txt")),
		Some(&mut AAA)
	);
	assert_eq!(
		root.get_child_mut(&crate::ItemPath::from("A/AA/")),
		Some(&mut AA)
	);
	assert_eq!(
		root.get_child_mut(&crate::ItemPath::from("A/AA")),
		Some(&mut AA)
	);
	assert_eq!(
		root.get_child_mut(&crate::ItemPath::from("A/")),
		Some(&mut A)
	);
	assert_eq!(
		root.get_child_mut(&crate::ItemPath::from("A")),
		Some(&mut A)
	);
	assert!(root.get_child_mut(&crate::ItemPath::from("")).is_some());

	assert_eq!(root.get_child_mut(&crate::ItemPath::from("B")), None);
	assert_eq!(root.get_child_mut(&crate::ItemPath::from("B/")), None);
	assert_eq!(root.get_child_mut(&crate::ItemPath::from("B/BB")), None);
	assert_eq!(root.get_child_mut(&crate::ItemPath::from("B/BB/")), None);
}

/// Check if a name of the part of a path does not contains unauthorized content.
///
/// Part of a path = the folder name, between an `/` and another `/`, or the filename.
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

	if path.contains('/') {
		return Err(format!("`{}` should not contains `/` character", path));
	}

	if path.contains('\\') {
		return Err(format!("`{}` should not contains `\\` character", path));
	}

	if path.contains('\0') {
		return Err(format!("`{}` should not contains `\\0` character", path));
	}

	return Ok(());
}

pub fn item_name_is_ok_without_itemdata(path: &str) -> Result<(), String> {
	if path.contains(".itemdata.") {
		return Err(format!(
			"`{}` should not contains `.itemdata.` string",
			path
		));
	}

	return item_name_is_ok(path);
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
		Err(format!("`{}` should not contains `\\0` character", input))
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

/// Simply store if user has `Read` or `ReadWrite` right to an endpoint
/// of [`Database`][`crate::database::Database`].
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

/// Errors can occurs when using [`TryFrom<&str>`][`std::convert::TryFrom`] on [`ScopeRightType`].
#[derive(Debug, PartialEq)]
pub enum ScopeParsingError {
	IncorrectFormat(String),
	IncorrectModule(String),
	IncorrectRight(String),
}

#[derive(PartialEq, Eq, Clone)]
pub struct ItemPath(Vec<ItemPathPart>);
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
			parts: &self.0,
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
			.unwrap_or(&crate::ItemPath::from("everything_else"))
			!= &crate::ItemPath::from("")
		{
			result.insert(0, crate::ItemPath::from(""));
		}

		return result;
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
impl std::convert::From<&ItemPath> for std::path::PathBuf {
	fn from(input: &ItemPath) -> Self {
		std::path::PathBuf::from(
			&format!("{}", input).replace('/', &String::from(std::path::MAIN_SEPARATOR)),
		)
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

#[test]
fn gu6qe5xy4zdl() {
	assert_eq!(
		ItemPath::from("a/b/c").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Document(String::from("c"))
		]
	);
}
#[test]
fn ojni817xbdfsv4lryfol3() {
	assert_eq!(
		ItemPath::from("a/b/c/").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Folder(String::from("c"))
		]
	);
}
#[test]
fn h60ujth6dopz1fbcg() {
	assert_eq!(
		ItemPath::from("a/b/../c/").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("c"))
		]
	);
}
#[test]
fn pwjqrwivvf4es31() {
	assert_eq!(
		ItemPath::from("..").0,
		vec![ItemPathPart::Folder(String::from("")),]
	);
}
#[test]
fn ccyet1ejsei14hxrmberswip() {
	assert_eq!(
		ItemPath::from("a\\b\\c").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Document(String::from("c"))
		]
	);
}
#[test]
fn tuxcgtsmdowij() {
	assert_eq!(
		ItemPath::from("a\\b\\c\\").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Folder(String::from("c"))
		]
	);
}
#[test]
fn jcavyevvgnm60mdlg2g12() {
	assert_eq!(
		ItemPath::from("a\\b\\..\\c\\").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("c"))
		]
	);
}
#[test]
fn v201kvbp5rkamp1m2u62gkd1() {
	assert_eq!(
		ItemPath::from("a\\b/c").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Document(String::from("c"))
		]
	);
}
#[test]
fn wprify4w2e82oaalbbxvjwi() {
	assert_eq!(
		ItemPath::from("").0,
		vec![ItemPathPart::Folder(String::from("")),]
	);
}
#[test]
fn ci40aqtbosaxg50cpl5z() {
	assert_eq!(
		ItemPath::from("/").0,
		vec![ItemPathPart::Folder(String::from("")),]
	);
}
#[test]
fn edtq0renvugb08s03j186ghn() {
	assert_eq!(
		ItemPath::from("/a/").0,
		vec![ItemPathPart::Folder(String::from("a")),]
	);
}

#[test]
fn p3ubrdxjoepapkt1h() {
	assert_eq!(
		ItemPath::from("./a/b").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Document(String::from("b"))
		]
	);
}

#[test]
fn qm2ek4irkfzrbwriz56() {
	assert_eq!(
		ItemPath::from("a/aa/aaa.txt").ancestors(),
		vec![
			ItemPath::from(""),
			ItemPath::from("a/"),
			ItemPath::from("a/aa/"),
			ItemPath::from("a/aa/aaa.txt"),
		]
	);
}

#[test]
fn vca2gwyljdba7r4xrv8hc386() {
	assert_eq!(
		ItemPath::from("a/aa/").ancestors(),
		vec![
			ItemPath::from(""),
			ItemPath::from("a/"),
			ItemPath::from("a/aa/"),
		]
	);
}

#[test]
fn kowoqexgrbp() {
	assert_eq!(ItemPath::from("").ancestors(), vec![ItemPath::from(""),]);
}

#[test]
fn hf40iqi11jtqn6hhqrxttbgj() {
	assert_eq!(ItemPath::from("/").ancestors(), vec![ItemPath::from(""),]);
}

#[test]
fn xes8rxrql76hb() {
	assert_eq!(
		ItemPath::from("")
			.joined(&crate::ItemPathPart::Folder(String::new()))
			.unwrap(),
		ItemPath::from("")
	);
}

#[test]
fn bp6kvtpdcyhu5ip8() {
	assert_eq!(
		ItemPath::from("")
			.joined(&crate::ItemPathPart::Folder(String::from("A")))
			.unwrap(),
		ItemPath::from("A/")
	);
}

#[test]
fn h8br2stuj50joa() {
	assert_eq!(
		ItemPath::from("")
			.joined(&crate::ItemPathPart::Folder(String::from("A")))
			.unwrap()
			.joined(&crate::ItemPathPart::Folder(String::new()))
			.unwrap()
			.joined(&crate::ItemPathPart::Folder(String::from("AA")))
			.unwrap(),
		ItemPath::from("A/AA/")
	);
}

#[test]
fn pqxxd8pzob0a8mk182hn() {
	assert_eq!(ItemPath::from("A/").parent(), Some(ItemPath::from("")));
}

#[test]
fn vuzxh45545c6pdbh7azm() {
	assert_eq!(ItemPath::from("").parent(), None);
}

impl From<&std::path::Path> for ItemPath {
	fn from(input: &std::path::Path) -> Self {
		ItemPath::from(&*input.to_string_lossy())
	}
}

#[test]
fn jmnk3j1xv8mq7cgvifwpz43() {
	assert_eq!(
		ItemPath::from(std::path::Path::new("a/b/c.txt")).0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Document(String::from("c.txt"))
		]
	);
}

#[test]
fn u2wb0ag5vhk0xhmd460() {
	assert_eq!(
		ItemPath::from(std::path::Path::new("a/b/c/")).0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Folder(String::from("c"))
		]
	);
}

#[test]
fn ap4x45tny4jekferziyr() {
	assert_eq!(
		ItemPath::from(std::path::Path::new("a\\b/c")).0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Document(String::from("c"))
		]
	);
}

#[test]
fn al7alq4uqj6mnnma1g() {
	assert_eq!(
		ItemPath::from(std::path::Path::new("C:\\a\\b")).0,
		vec![
			ItemPathPart::Folder(String::from("C:")),
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Document(String::from("b"))
		]
	);
}

#[test]
fn hr1lrd0v64hrpypoq() {
	assert_eq!(
		ItemPath::from(std::path::Path::new("\\\\SERVER\\a\\b")).0,
		vec![
			ItemPathPart::Folder(String::from("SERVER")),
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Document(String::from("b"))
		]
	);
}

#[test]
fn bjwt3pbcft0gacq284s() {
	assert_eq!(
		ItemPath::from(std::path::Path::new("\\a\\\\b\\c")).0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Document(String::from("c"))
		]
	);
}

#[derive(PartialEq, Eq, Clone)]
pub enum ItemPathPart {
	Folder(String),
	Document(String),
}
impl std::fmt::Display for ItemPathPart {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Folder(name) => f.write_fmt(format_args!("{}/", name)),
			Self::Document(name) => f.write_str(name),
		}
	}
}
impl ItemPathPart {
	fn name(&self) -> &str {
		match self {
			Self::Folder(name) => name,
			Self::Document(name) => name,
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

pub struct ItemPathPartIterator<'a> {
	parts: &'a Vec<ItemPathPart>,
	current_pos: usize,
}
impl<'a> Iterator for ItemPathPartIterator<'a> {
	type Item = &'a ItemPathPart;
	fn next(&mut self) -> Option<Self::Item> {
		let result = self.parts.get(self.current_pos);
		self.current_pos += 1;

		return result;
	}
}

pub struct ItemPathAncestorsIterator<'a> {
	ancestors: &'a [ItemPath],
	current_pos: usize,
}
impl<'a> Iterator for ItemPathAncestorsIterator<'a> {
	type Item = &'a ItemPath;
	fn next(&mut self) -> Option<Self::Item> {
		let result = self.ancestors.get(self.current_pos);
		self.current_pos += 1;

		return result;
	}
}
