mod api;
mod tokens;
mod webfinger;

mod database;
mod https;
mod logger;
mod settings;
mod users;

pub use database::load_or_create_database;
pub use https::setup_and_run_https_server;
pub use logger::load_or_create_logger;
pub use settings::{load_or_create_settings, Settings, SettingsHTTPS};
pub use users::load_or_create_users;

pub mod middlewares;

pub use api::*;
pub use tokens::*;
pub use webfinger::webfinger_handle;

use rand::seq::IteratorRandom;
use rand::Rng;

pub const RFC5322: &str = "%a, %d %b %Y %H:%M:%S %Z";
const FORM_TOKEN_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ?,;.:/!§*µù%$£¤=+{}[]()°à@çè|#é~&";
const PASSWORD_HASH_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ?,;.:/!§*µù%$£¤=+{}[]()°à@çè|#é~&";
const ACCESS_TOKEN_ALPHABET: &str =
	"abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ!+*";

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
}
impl Users {
	// TODO : tests
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
}
/*
impl Users {
	pub fn get_usernames(&self) -> Vec<&String> {
		self.list.iter().map(|user| &user.name).collect()
	}
}
*/
impl Users {
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

#[derive(serde::Deserialize, serde::Serialize)]
struct User {
	name: String,
	rights: Vec<UserRight>,
	hashed_password: Vec<u8>,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum UserRight {
	ManageUsers,
}
impl std::fmt::Display for UserRight {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::ManageUsers => f.write_str("manage users"),
		}
	}
}

#[actix_web::get("/favicon.ico")]
pub async fn favicon() -> actix_web::web::HttpResponse {
	return actix_web::HttpResponse::Ok().body(actix_web::web::Bytes::from_static(include_bytes!(
		"static/favicon.ico"
	)));
}

#[actix_web::get("/remotestorage.svg")]
pub async fn remotestoragesvg() -> actix_web::web::HttpResponse {
	return actix_web::HttpResponse::Ok().body(actix_web::web::Bytes::from_static(include_bytes!(
		"static/remotestorage.svg"
	)));
}

#[actix_web::get("/")]
pub async fn index() -> actix_web::web::HttpResponse {
	actix_web::HttpResponse::Ok().body(format!(
		r#"<!DOCTYPE html>
<html>
	<head>
		<meta charset="UTF-8">
		<meta http-equiv="X-UA-Compatible" content="IE=edge">
		<meta name="viewport" content="width=device-width, initial-scale=1.0">
		<title>{}</title>
	</head>
	<body style="padding:1em 2em;">
		<h1>
			<img src="/favicon.ico" alt="" style="max-height:2em;vertical-align:middle;">
			{} V{}
		</h1>
		<p>
			This is a
			&nbsp;
			<a href="https://remotestorage.io/">
				<img src="/remotestorage.svg" style="max-height:1em;vertical-align:middle;">
				remoteStorage
			</a>
			server.
		</p>
		<p>
			Find Apps compatible with this database
			<a href="https://wiki.remotestorage.io/Apps">on remotestorage wiki</a>
			or
			<a href="https://0data.app/">on 0data list</a>
			.
		</p>
		<hr>
		<p>
			See source code on
			<a href="https://github.com/Jimskapt/pontus_onyx">GitHub</a>.
		</p>
		<p>
			Made with ❤ by
			<a href="https://jimskapt.com/">Thomas RAMIREZ</a> in France 🇫🇷
		</p>
	</body>
</html>"#,
		env!("CARGO_PKG_NAME"),
		env!("CARGO_PKG_NAME"),
		env!("CARGO_PKG_VERSION")
	))
}

pub fn build_server_address(
	settings: &crate::http_server::Settings,
	program_state: &crate::ProgramState,
) -> String {
	let mut protocol = String::from("http");
	if let Some(force_https) = settings.force_https {
		if force_https {
			protocol += "s";
		}
	} else if program_state.https_mode {
		protocol += "s";
	}

	let mut domain = String::from("localhost");
	if let Some(force_domain) = &settings.domain {
		if !force_domain.trim().is_empty() {
			domain = force_domain.clone();
		}
	}

	let port = if let Some(force_https) = &settings.force_https {
		if *force_https {
			if let Some(https) = &settings.https {
				if https.port != 443 {
					format!(":{}", https.port)
				} else {
					String::new()
				}
			} else if settings.port != 80 {
				format!(":{}", settings.port)
			} else {
				String::new()
			}
		} else if program_state.https_mode {
			let https = settings.https.clone().unwrap();
			if https.port != 443 {
				format!(":{}", https.port)
			} else {
				String::new()
			}
		} else if settings.port != 80 {
			format!(":{}", settings.port)
		} else {
			String::new()
		}
	} else if program_state.https_mode {
		let https = settings.https.clone().unwrap();
		if https.port != 443 {
			format!(":{}", https.port)
		} else {
			String::new()
		}
	} else if settings.port != 80 {
		format!(":{}", settings.port)
	} else {
		String::new()
	};

	let mut domain_suffix = String::new();
	if let Some(suffix) = &settings.domain_suffix {
		if !suffix.trim().is_empty() && !suffix.trim().ends_with('/') {
			domain_suffix = format!("{}/", suffix.trim())
		} else {
			domain_suffix = String::from(suffix.trim())
		}
	}

	format!("{}://{}{}/{}", protocol, domain, port, domain_suffix)
}

#[test]
fn pbw1cgzctiqe163() {
	let mut settings = Settings::default();
	let state = crate::ProgramState { https_mode: false };

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}/{}",
			"http",
			format!("localhost:{}", settings.port),
			""
		)
	);
}

#[test]
fn ykf0gcnr7z2ko4wtx8uub() {
	let mut settings = Settings::default();
	settings.domain_suffix = Some(String::from("test"));
	let state = crate::ProgramState { https_mode: false };

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}/{}",
			"http",
			format!("localhost:{}", settings.port),
			"test/"
		)
	);
}

#[test]
fn wxpy6tncuwbbavvxi() {
	let mut settings = Settings::default();
	settings.domain_suffix = Some(String::from("test/"));
	let state = crate::ProgramState { https_mode: false };

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}/{}",
			"http",
			format!("localhost:{}", settings.port),
			"test/"
		)
	);
}

#[test]
fn fpfxwrixa1jz7t() {
	let mut settings = Settings::default();
	let state = crate::ProgramState { https_mode: true };

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}/{}",
			"https",
			format!("localhost:{}", settings.https.unwrap().port),
			""
		)
	);
}

#[test]
fn xtgfpc3x1zcmb() {
	let domain = String::from("example.com");
	let mut settings = Settings::default();
	settings.domain = Some(domain.clone());
	let state = crate::ProgramState { https_mode: false };

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}/{}",
			"http",
			format!("{}:{}", domain, settings.port),
			""
		)
	);
}

#[test]
fn ekkvpuijzifxc() {
	let domain = String::from("example.com");
	let mut settings = Settings::default();
	settings.domain = Some(domain.clone());
	let state = crate::ProgramState { https_mode: true };

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}/{}",
			"https",
			format!("{}:{}", domain, settings.https.unwrap().port),
			""
		)
	);
}

#[test]
fn bj8n5zhu2oaaed55561ygk() {
	let domain = String::from("example.com");
	let mut settings = Settings::default();
	settings.domain = Some(domain.clone());
	settings.port = 80;
	if let Some(https) = &mut settings.https {
		https.port = 443;
	}
	let state = crate::ProgramState { https_mode: false };

	assert_eq!(
		build_server_address(&settings, &state),
		format!("{}://{}/{}", "http", domain, "")
	);
}

#[test]
fn d434yaaxfqcnd4j() {
	let domain = String::from("example.com");
	let mut settings = Settings::default();
	settings.domain = Some(domain.clone());
	settings.port = 80;
	if let Some(https) = &mut settings.https {
		https.port = 443;
	}
	let state = crate::ProgramState { https_mode: true };

	assert_eq!(
		build_server_address(&settings, &state),
		format!("{}://{}/{}", "https", domain, "")
	);
}
