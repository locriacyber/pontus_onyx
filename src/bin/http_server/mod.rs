mod api;
mod init;
mod tokens;
mod users;
mod utils;
mod webfinger;

pub mod middlewares;

pub use api::*;
pub use init::*;
pub use tokens::*;
pub use webfinger::webfinger_handle;

use users::*;
use utils::build_server_address;

use std::sync::{Arc, Mutex};

pub const RFC5322: &str = "%a, %d %b %Y %H:%M:%S %Z";
const FORM_TOKEN_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ?,;.:/!§*µù%$£¤=+{}[]()°à@çè|#é~&";
const PASSWORD_HASH_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ?,;.:/!§*µù%$£¤=+{}[]()°à@çè|#é~&";
const ACCESS_TOKEN_ALPHABET: &str =
	"abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ!+*";

pub fn configure_server(
	settings: Arc<Mutex<crate::http_server::Settings>>,
	database: Arc<Mutex<pontus_onyx::database::Database>>,
	access_tokens: Arc<Mutex<Vec<crate::http_server::AccessBearer>>>,
	oauth_form_tokens: Arc<Mutex<Vec<crate::http_server::middlewares::OauthFormToken>>>,
	users: Arc<Mutex<crate::http_server::Users>>,
	program_state: Arc<Mutex<crate::ProgramState>>,
	logger: Arc<Mutex<charlie_buffalo::Logger>>,
) -> impl FnOnce(&mut actix_web::web::ServiceConfig) {
	return move |config: &mut actix_web::web::ServiceConfig| {
		config
			.data(database.clone())
			.data(oauth_form_tokens.clone())
			.data(access_tokens.clone())
			.data(users.clone())
			.data(settings.clone())
			.data(program_state.clone())
			.data(logger)
			.service(favicon)
			.service(get_oauth)
			.service(post_oauth)
			.service(webfinger_handle)
			.service(get_item)
			.service(head_item)
			.service(options_item)
			.service(put_item)
			.service(delete_item)
			.service(remotestoragesvg)
			.service(index);
	};
}

#[actix_web::get("/favicon.ico")]
pub async fn favicon() -> impl actix_web::Responder {
	return actix_web::HttpResponse::Ok().body(actix_web::web::Bytes::from_static(include_bytes!(
		"static/favicon.ico"
	)));
}

#[actix_web::get("/remotestorage.svg")]
pub async fn remotestoragesvg() -> impl actix_web::Responder {
	return actix_web::HttpResponse::Ok().body(actix_web::web::Bytes::from_static(include_bytes!(
		"static/remotestorage.svg"
	)));
}

#[actix_web::get("/")]
pub async fn index() -> impl actix_web::Responder {
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
