#[derive(serde::Deserialize, serde::Serialize)]
pub struct User {
	pub name: String,
	pub rights: Vec<UserRight>,
	pub hashed_password: Vec<u8>,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum UserRight {
	ManageServerSettings,
	ManageUsers,
	ManageApplications,
}
impl std::fmt::Display for UserRight {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::ManageServerSettings => f.write_str("manage server settings"),
			Self::ManageUsers => f.write_str("manage users"),
			Self::ManageApplications => f.write_str("manage applications"),
		}
	}
}
