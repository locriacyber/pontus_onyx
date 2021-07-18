use rand::seq::IteratorRandom;
use rand::Rng;

#[derive(Debug, PartialEq, Clone)]
pub struct Scope {
	pub right_type: pontus_onyx::ScopeRightType,
	pub module: String,
}
impl Scope {
	pub fn allowed_methods(&self) -> Vec<actix_web::http::Method> {
		match self.right_type {
			pontus_onyx::ScopeRightType::Read => vec![
				actix_web::http::Method::GET,
				actix_web::http::Method::HEAD,
				actix_web::http::Method::OPTIONS,
			],
			pontus_onyx::ScopeRightType::ReadWrite => vec![
				actix_web::http::Method::GET,
				actix_web::http::Method::HEAD,
				actix_web::http::Method::PUT,
				actix_web::http::Method::DELETE,
				actix_web::http::Method::OPTIONS,
			],
		}
	}
}
impl std::convert::TryFrom<&str> for Scope {
	type Error = pontus_onyx::ScopeParsingError;

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
							return Err(pontus_onyx::ScopeParsingError::IncorrectModule(
								String::from(module),
							));
						}

						let regex = regex::Regex::new("^[a-z0-9_]+$").unwrap();
						if module == "*" || regex.is_match(module) {
							let right_type = pontus_onyx::ScopeRightType::try_from(right)?;
							let module = String::from(module);

							Ok(Self { right_type, module })
						} else {
							Err(pontus_onyx::ScopeParsingError::IncorrectModule(
								String::from(module),
							))
						}
					}
					None => Err(pontus_onyx::ScopeParsingError::IncorrectFormat(
						String::from(input),
					)),
				},
				None => Err(pontus_onyx::ScopeParsingError::IncorrectFormat(
					String::from(input),
				)),
			},
			Some(_) => {
				return Err(pontus_onyx::ScopeParsingError::IncorrectFormat(
					String::from(input),
				));
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct AccessBearer {
	name: String,
	scopes: Vec<Scope>,
	client_id: String,
	username: String,
	emit_time: std::time::Instant,
}
impl AccessBearer {
	pub fn new(scopes: Vec<Scope>, client_id: &str, username: &str) -> Self {
		let mut name = String::new();

		let mut rng_limit = rand::thread_rng();
		for _ in 1..rng_limit.gen_range(128..256) {
			let mut rng_item = rand::thread_rng();
			name.push(
				crate::http_server::ACCESS_TOKEN_ALPHABET
					.chars()
					.choose(&mut rng_item)
					.unwrap(),
			);
		}
		name.push('=');

		Self {
			name,
			scopes,
			client_id: String::from(client_id),
			username: String::from(username),
			emit_time: std::time::Instant::now(),
		}
	}

	pub fn get_name(&self) -> &str {
		&self.name
	}
	pub fn get_scopes(&self) -> &[Scope] {
		&self.scopes
	}
	/*
	pub fn get_client_id(&self) -> &str {
		&self.client_id
	}
	*/
	pub fn get_username(&self) -> &str {
		&self.username
	}
	pub fn get_emit_time(&self) -> &std::time::Instant {
		&self.emit_time
	}
}

#[cfg(test)]
mod tests {
	use std::convert::TryFrom;

	#[test]
	fn c0ok0eil7m3() {
		assert_eq!(
			super::Scope::try_from("*:rw"),
			Ok(super::Scope {
				right_type: pontus_onyx::ScopeRightType::ReadWrite,
				module: String::from("*")
			})
		);
	}

	#[test]
	fn kn76nin3ppdf25t3p7zao() {
		assert_eq!(
			super::Scope::try_from("random:r"),
			Ok(super::Scope {
				right_type: pontus_onyx::ScopeRightType::Read,
				module: String::from("random")
			})
		);
	}

	#[test]
	fn sllj3xshcq266faixwpa() {
		assert_eq!(
			super::Scope::try_from("public:rw"),
			Err(pontus_onyx::ScopeParsingError::IncorrectModule(
				String::from("public")
			))
		);
	}

	#[test]
	fn mt1ns651q04kfc() {
		assert_eq!(
			super::Scope::try_from("wrong_char@:rw"),
			Err(pontus_onyx::ScopeParsingError::IncorrectModule(
				String::from("wrong_char@")
			))
		);
	}

	#[test]
	fn gy8bajrpald87() {
		assert_eq!(
			super::Scope::try_from("wrong:char:rw"),
			Err(pontus_onyx::ScopeParsingError::IncorrectFormat(
				String::from("wrong:char:rw")
			))
		);
	}

	#[test]
	fn jrx3s6biaha6ztxvollgn() {
		assert_eq!(
			super::Scope::try_from("random:wrong_right"),
			Err(pontus_onyx::ScopeParsingError::IncorrectRight(
				String::from("wrong_right")
			))
		);
	}
}
