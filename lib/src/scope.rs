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

#[derive(Debug, PartialEq, Clone)]
pub struct Scope {
	pub right_type: ScopeRightType,
	pub module: String,
}
#[cfg(feature = "server")]
impl Scope {
	pub fn allowed_methods(&self) -> Vec<actix_web::http::Method> {
		match self.right_type {
			ScopeRightType::Read => vec![
				actix_web::http::Method::GET,
				actix_web::http::Method::HEAD,
				actix_web::http::Method::OPTIONS,
			],
			ScopeRightType::ReadWrite => vec![
				actix_web::http::Method::GET,
				actix_web::http::Method::HEAD,
				actix_web::http::Method::PUT,
				actix_web::http::Method::DELETE,
				actix_web::http::Method::OPTIONS,
			],
		}
	}
	pub fn is_allowed(
		&self,
		method: &actix_web::http::Method,
		path: impl Into<String>,
		username: impl Into<String>,
	) -> bool {
		if self
			.allowed_methods()
			.iter()
			.any(|allowed_method| allowed_method == method)
		{
			let path = path.into();
			let username = username.into();

			if self.module == "*" {
				path.starts_with("/storage/")
			} else {
				path.starts_with(&format!("/storage/{}/{}", username, self.module))
					|| path.starts_with(&format!("/storage/public/{}/{}", username, self.module))
					|| path.starts_with("/events/")
			}
		} else {
			false
		}
	}
}
impl std::convert::TryFrom<&str> for Scope {
	type Error = ScopeParsingError;

	fn try_from(input: &str) -> Result<Self, Self::Error> {
		let mut temp = input.split(':');

		let module = temp.next();
		let right = temp.next();
		let remaining = temp.next();

		match remaining {
			None => match module {
				Some(module) => match right {
					Some(right) => {
						if module == "public" {
							return Err(ScopeParsingError::IncorrectModule(String::from(module)));
						}

						let regex = regex::Regex::new("^[a-z0-9_]+$").unwrap();
						if module == "*" || regex.is_match(module) {
							let right_type = ScopeRightType::try_from(right)?;
							let module = String::from(module);

							Ok(Self { right_type, module })
						} else {
							Err(ScopeParsingError::IncorrectModule(String::from(module)))
						}
					}
					None => Err(ScopeParsingError::IncorrectFormat(String::from(input))),
				},
				None => Err(ScopeParsingError::IncorrectFormat(String::from(input))),
			},
			Some(_) => {
				return Err(ScopeParsingError::IncorrectFormat(String::from(input)));
			}
		}
	}
}
impl std::fmt::Display for Scope {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		f.write_fmt(format_args!(
			"{}:{}",
			self.module,
			match self.right_type {
				ScopeRightType::Read => "r",
				ScopeRightType::ReadWrite => "rw",
			}
		))
	}
}
