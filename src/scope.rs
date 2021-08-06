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
