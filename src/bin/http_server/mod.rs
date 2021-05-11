mod api;
mod middlewares;
mod tokens;
mod webfinger;

pub use api::*;
pub use middlewares::*;
pub use tokens::*;
pub use webfinger::webfinger_handle;

use rand::seq::IteratorRandom;
use rand::Rng;

pub const RFC5322: &str = "%a, %d %b %Y %H:%M:%S %Z";
const FORM_TOKEN_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ?,;.:/!§*µù%$£¤=+{}[]()°à@çè|#é~&";
const PASSWORD_HASH_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ?,;.:/!§*µù%$£¤=+{}[]()°à@çè|#é~&";
const ACCESS_TOKEN_ALPHABET: &str =
	"abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ!+*";

pub struct Users {
	salt: String,
	list: std::collections::HashMap<String, Vec<u8>>,
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

		Self {
			salt,
			list: std::collections::HashMap::new(),
		}
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

		match self.list.get(username) {
			Some(user_password) => user_password == &hashed_password,
			None => false,
		}
	}
}
impl Users {
	pub fn insert(&mut self, username: &str, password: &mut String) {
		let mut hasher = hmac_sha512::Hash::new();
		hasher.update(self.salt.as_bytes());
		hasher.update(password.as_bytes());
		hasher.update(self.salt.as_bytes());

		zeroize::Zeroize::zeroize(password);

		self.list
			.insert(String::from(username), hasher.finalize().to_vec());
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
	actix_web::HttpResponse::Ok()
		.body(format!(r#"<!DOCTYPE html>
<html>
	<head>
		<meta charset="UTF-8">
		<meta http-equiv="X-UA-Compatible" content="IE=edge">
		<meta name="viewport" content="width=device-width, initial-scale=1.0">
		<title>{}</title>
	</head>
	<body style="padding:1em 2em;">
		<h1><img src="/favicon.ico" alt="" style="max-height:2em;vertical-align:middle;"> {}</h1>
		<p>This is an <a href="https://remotestorage.io/"><img src="/remotestorage.svg" style="max-height:1em;vertical-align:middle;"> remoteStorage</a> server.</p>
		<p>Find Apps compatible with this database <a href="https://wiki.remotestorage.io/Apps">here</a> or <a href="https://0data.app/">here</a>.</p>
		<p>See source code on <a href="https://github.com/Jimskapt/pontus_onyx">GitHub</a>.</p>
	</body>
</html>"#, env!("CARGO_PKG_NAME"), env!("CARGO_PKG_NAME")))
}
