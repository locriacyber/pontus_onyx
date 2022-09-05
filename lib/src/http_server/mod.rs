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

const FORM_TOKEN_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ?,;.:/!§*µù%$£¤=+{}[]()°à@çè|#é~&";
const PASSWORD_HASH_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ?,;.:/!§*µù%$£¤=+{}[]()°à@çè|#é~&";
const ACCESS_TOKEN_ALPHABET: &str =
	"abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ!+*";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DbEvent {
	id: String,
	date: time::OffsetDateTime,
	method: DbEventMethod,
	path: String,
	etag: crate::item::Etag,
	user: String,
	dbversion: String,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub enum DbEventMethod {
	Put,
	Delete,
}

pub fn configure_server(
	settings: Arc<Mutex<crate::http_server::Settings>>,
	database: Arc<Mutex<crate::database::Database>>,
	access_tokens: Arc<Mutex<Vec<crate::http_server::AccessBearer>>>,
	oauth_form_tokens: Arc<Mutex<Vec<crate::http_server::middlewares::OauthFormToken>>>,
	users: Arc<Mutex<crate::http_server::Users>>,
	program_state: Arc<Mutex<ProgramState>>,
	logger: Arc<Mutex<charlie_buffalo::Logger>>,
	dbevent_sender: Option<std::sync::mpsc::Sender<DbEvent>>,
) -> impl FnOnce(&mut actix_web::web::ServiceConfig) {
	return move |config: &mut actix_web::web::ServiceConfig| {
		config
			.app_data(actix_web::web::Data::new(database.clone()))
			.app_data(actix_web::web::Data::new(oauth_form_tokens.clone()))
			.app_data(actix_web::web::Data::new(access_tokens.clone()))
			.app_data(actix_web::web::Data::new(users.clone()))
			.app_data(actix_web::web::Data::new(settings.clone()))
			.app_data(actix_web::web::Data::new(program_state.clone()))
			.app_data(actix_web::web::Data::new(logger));

		if let Some(dbevent_sender) = dbevent_sender {
			config.app_data(actix_web::web::Data::new(dbevent_sender.clone()));
		}

		config
			.service(options_favicon)
			.service(get_favicon)
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

#[actix_web::options("/favicon.ico")]
pub async fn options_favicon() -> impl actix_web::Responder {
	let mut res = actix_web::HttpResponse::Ok();
	res.insert_header((actix_web::http::header::ALLOW, "OPTIONS, GET"));
	res.insert_header((actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"));

	return res;
}

#[actix_web::get("/favicon.ico")]
pub async fn get_favicon() -> impl actix_web::Responder {
	let mut res = actix_web::HttpResponse::Ok();
	res.insert_header((actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"));

	return res.body(actix_web::web::Bytes::from_static(crate::assets::ICON));
}

#[actix_web::get("/remotestorage.svg")]
pub async fn remotestoragesvg() -> impl actix_web::Responder {
	return actix_web::HttpResponse::Ok().body(actix_web::web::Bytes::from_static(
		crate::assets::REMOTE_STORAGE,
	));
}

#[actix_web::get("/")]
pub async fn index() -> impl actix_web::Responder {
	let template: &str = include_str!("./static/index.html");
	let template = template.replace("{{app_name}}", env!("CARGO_PKG_NAME"));
	let template = template.replace("{{app_version}}", env!("CARGO_PKG_VERSION"));

	actix_web::HttpResponse::Ok().body(template)
}

#[derive(Debug, Clone, Default)]
pub struct ProgramState {
	pub https_mode: bool,
}
