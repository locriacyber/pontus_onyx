/// Used to versioning purposes.
///
/// It is a String value.
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
