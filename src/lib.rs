#![allow(clippy::needless_return)]
#![allow(non_snake_case)]

#[cfg(feature = "client_lib")]
pub mod client;

#[cfg(feature = "server_lib")]
pub mod database;

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

#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[derivative(Debug)]
pub enum Item {
	Folder {
		etag: crate::Etag,
		#[derivative(Debug = "ignore")]
		content: Option<std::collections::HashMap<String, Box<crate::Item>>>,
	},
	Document {
		etag: crate::Etag,
		#[derivative(Debug = "ignore")]
		content: Option<Vec<u8>>,
		content_type: crate::ContentType,
		last_modified: chrono::DateTime<chrono::offset::Utc>,
	},
}
impl Item {
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

	pub fn new_doc(content: &[u8], content_type: &str) -> Self {
		return Self::Document {
			etag: crate::Etag::new(),
			content: Some(content.to_vec()),
			content_type: crate::ContentType::from(content_type),
			last_modified: chrono::Utc::now(),
		};
	}

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

	pub fn is_folder(&self) -> bool {
		match self {
			Self::Folder { .. } => true,
			Self::Document { .. } => false,
		}
	}

	pub fn is_document(&self) -> bool {
		match self {
			Self::Document { .. } => true,
			Self::Folder { .. } => false,
		}
	}

	pub fn get_etag(&self) -> &crate::Etag {
		return match self {
			Self::Folder { etag, .. } => etag,
			Self::Document { etag, .. } => etag,
		};
	}

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
		root.get_child(&std::path::PathBuf::from("A/AA/AAA.txt")),
		Some(&AAA)
	);
	assert_eq!(
		root.get_child(&std::path::PathBuf::from("A/AA/")),
		Some(&AA)
	);
	assert_eq!(root.get_child(&std::path::PathBuf::from("A/AA")), Some(&AA));
	assert_eq!(root.get_child(&std::path::PathBuf::from("A/")), Some(&A));
	assert_eq!(root.get_child(&std::path::PathBuf::from("A")), Some(&A));
	assert_eq!(root.get_child(&std::path::PathBuf::from("")), Some(&root));

	assert_eq!(root.get_child(&std::path::PathBuf::from("B")), None);
	assert_eq!(root.get_child(&std::path::PathBuf::from("B/")), None);
	assert_eq!(root.get_child(&std::path::PathBuf::from("B/BB")), None);
	assert_eq!(root.get_child(&std::path::PathBuf::from("B/BB/")), None);
}

#[test]
fn j5bmhdxhlgkhdk82rjio3ej6() {
	let mut AAA = crate::Item::new_doc(b"test", "text/plain");
	let mut AA = crate::Item::new_folder(vec![("AAA.txt", AAA.clone())]);
	let mut A = crate::Item::new_folder(vec![("AA", AA.clone())]);
	let mut root = crate::Item::new_folder(vec![("A", A.clone())]);

	assert_eq!(
		root.get_child_mut(&std::path::PathBuf::from("A/AA/AAA.txt")),
		Some(&mut AAA)
	);
	assert_eq!(
		root.get_child_mut(&std::path::PathBuf::from("A/AA/")),
		Some(&mut AA)
	);
	assert_eq!(
		root.get_child_mut(&std::path::PathBuf::from("A/AA")),
		Some(&mut AA)
	);
	assert_eq!(
		root.get_child_mut(&std::path::PathBuf::from("A/")),
		Some(&mut A)
	);
	assert_eq!(
		root.get_child_mut(&std::path::PathBuf::from("A")),
		Some(&mut A)
	);
	assert!(root.get_child_mut(&std::path::PathBuf::from("")).is_some());

	assert_eq!(root.get_child_mut(&std::path::PathBuf::from("B")), None);
	assert_eq!(root.get_child_mut(&std::path::PathBuf::from("B/")), None);
	assert_eq!(root.get_child_mut(&std::path::PathBuf::from("B/BB")), None);
	assert_eq!(root.get_child_mut(&std::path::PathBuf::from("B/BB/")), None);
}
