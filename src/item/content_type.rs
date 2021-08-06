/// Used to indicate what kind of data is (binary) content of
/// an [`Document`][`crate::item::Item::Document`].
///
/// It is a String value.
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
