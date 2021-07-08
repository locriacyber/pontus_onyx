mod user;

use user::User;
pub use user::UserRight;

use rand::seq::IteratorRandom;
use rand::Rng;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Users {
	salt: String,
	list: Vec<User>,
}
impl Users {
	pub fn new() -> Self {
		let mut salt = String::new();
		let mut rng_limit = rand::thread_rng();
		for _ in 1..rng_limit.gen_range(16..32) {
			let mut rng_item = rand::thread_rng();
			salt.push(
				crate::http_server::PASSWORD_HASH_ALPHABET
					.chars()
					.choose(&mut rng_item)
					.unwrap(),
			);
		}

		Self { salt, list: vec![] }
	}

	pub fn check(&self, username: &str, password: &mut String) -> bool {
		let mut hasher = hmac_sha512::Hash::new();
		hasher.update(self.salt.as_bytes());
		hasher.update(password.as_bytes());
		hasher.update(self.salt.as_bytes());
		let hashed_password = hasher.finalize().to_vec();

		zeroize::Zeroize::zeroize(password);

		match self.list.iter().find(|user| user.name == username) {
			Some(user) => user.hashed_password == hashed_password,
			None => false,
		}
	}

	/*
	pub fn get_usernames(&self) -> Vec<&String> {
		self.list.iter().map(|user| &user.name).collect()
	}
	*/

	pub fn insert(&mut self, username: &str, password: &mut String) -> Result<(), String> {
		let mut hasher = hmac_sha512::Hash::new();
		hasher.update(self.salt.as_bytes());
		hasher.update(password.as_bytes());
		hasher.update(self.salt.as_bytes());

		zeroize::Zeroize::zeroize(password);

		if self.list.iter().any(|user| user.name == username) {
			return Err(String::from("this username already exists"));
		}

		self.list.push(User {
			name: String::from(username),
			rights: vec![],
			hashed_password: hasher.finalize().to_vec(),
		});

		return Ok(());
	}

	pub fn add_right(&mut self, username: &str, right: UserRight) -> Result<(), String> {
		match self.list.iter_mut().find(|user| user.name == username) {
			Some(user) => {
				if !user.rights.contains(&right) {
					user.rights.push(right);
					Ok(())
				} else {
					Err(String::from("user have already this right"))
				}
			}
			None => Err(String::from("user not found")),
		}
	}

	/*
	pub fn remove_right(&mut self, username: &str, right: UserRight) -> Result<(), String> {
		match self.list.iter_mut().find(|user| user.name == username) {
			Some(user) => match user.rights.binary_search(&right) {
				Ok(position) => {
					user.rights.remove(position);
					Ok(())
				}
				Err(_) => Err(String::from("user does not have already this right")),
			},
			None => Err(String::from("user not found")),
		}
	}
	*/
}

#[test]
fn qsaeipdjit2zwcqlpx() {
	let mut users = Users::new();
	assert_eq!(users.insert("user", &mut String::from("password")), Ok(()));
	assert_eq!(
		users.insert("user", &mut String::from("password")),
		Err(String::from("this username already exists"))
	);
}

#[test]
fn w0rls5kmnz() {
	let mut users = Users::new();
	assert_eq!(users.insert("user", &mut String::from("password")), Ok(()));
	assert_eq!(
		users.add_right("RANDOM", UserRight::ManageUsers),
		Err(String::from("user not found"))
	);
	assert_eq!(users.add_right("user", UserRight::ManageUsers), Ok(()));
	assert_eq!(
		users.add_right("user", UserRight::ManageUsers),
		Err(String::from("user have already this right"))
	);
}
