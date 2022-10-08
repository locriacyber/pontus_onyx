mod api;
mod init;
mod tokens;
mod users;
mod utils;
mod webfinger;

use users::*;
use utils::build_server_address;

use std::sync::{Arc, Mutex};

pub mod middlewares;

pub use api::*;
pub use init::*;
pub use tokens::*;
pub use users::Users;
pub use webfinger::webfinger_handle;

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
	Create,
	Update,
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
	workspace_path: &std::path::Path,
	dbevent_sender: Option<std::sync::mpsc::Sender<DbEvent>>,
) -> impl FnOnce(&mut actix_web::web::ServiceConfig) {
	let workspace_path_clone = workspace_path.to_path_buf();

	return move |config: &mut actix_web::web::ServiceConfig| {
		config
			.app_data(actix_web::web::Data::new(database.clone()))
			.app_data(actix_web::web::Data::new(oauth_form_tokens.clone()))
			.app_data(actix_web::web::Data::new(access_tokens.clone()))
			.app_data(actix_web::web::Data::new(users.clone()))
			.app_data(actix_web::web::Data::new(settings.clone()))
			.app_data(actix_web::web::Data::new(program_state.clone()))
			.app_data(actix_web::web::Data::new(workspace_path_clone))
			.app_data(actix_web::web::Data::new(logger));

		if let Some(dbevent_sender) = dbevent_sender {
			config.app_data(actix_web::web::Data::new(dbevent_sender));
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
			.service(server_events)
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

#[actix_web::get("/events/all")]
pub async fn server_events(
	workspace_path: actix_web::web::Data<std::path::PathBuf>,
	logger: actix_web::web::Data<Arc<Mutex<charlie_buffalo::Logger>>>,
	tokens: actix_web::web::Data<
		std::sync::Arc<std::sync::Mutex<Vec<crate::http_server::AccessBearer>>>,
	>,
	request: actix_web::HttpRequest,
	settings: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<crate::http_server::Settings>>>,
) -> impl actix_web::Responder {
	let mut file_content = std::fs::read(workspace_path.join("events.bin")).unwrap();
	file_content = file_content[..(file_content.len() - b",\n".len())].to_vec();
	let events: Vec<DbEvent> = {
		let mut file_content_stream = vec![b'['];
		file_content_stream.append(&mut file_content);
		file_content_stream.push(b']');

		let file_content_stream = String::from_utf8_lossy(&file_content_stream);
		let file_content_stream =
			file_content_stream.replace(r#""method":"Put""#, r#""method":"Update""#); // backward compatibility

		match serde_json::from_str(&file_content_stream) {
			Ok(events) => events,
			Err(err) => {
				logger.lock().unwrap().push(
					vec![
						(String::from("level"), String::from("ERROR")),
						(String::from("module"), String::from("https?")),
						(String::from("method"), String::from("GET")),
						(String::from("path"), String::from("/events/all")),
					],
					Some(&err.to_string()),
				);

				vec![]
			}
		}
	};

	let mut last_event_id = request
		.headers()
		.iter()
		.find(|(name, _)| name.as_str().trim().to_lowercase() == "last-event-id")
		.map(|(_, value)| String::from(value.to_str().unwrap()));
	if let Some(value) = &last_event_id {
		if value.is_empty() || value.trim().to_lowercase() == "null" {
			last_event_id = None;
		}
	}

	let token = request
		.headers()
		.iter()
		.find(|(name, _)| name.as_str().trim().to_lowercase() == "authorization")
		.map(|(_, value)| value.to_str().unwrap())
		.map(|value| match value.strip_prefix("Bearer ") {
			Some(value) => String::from(value),
			None => String::from("error"),
		});

	let token = token.and_then(|token| {
		tokens
			.lock()
			.unwrap()
			.iter()
			.find(|bearer| bearer.get_name() == token)
			.cloned()
	});

	let mut reponse_content = String::new();
	let mut id_found = last_event_id.is_none();
	let max_token_lifetime_seconds = settings
		.lock()
		.unwrap()
		.token_lifetime_seconds
		.unwrap_or_else(|| {
			crate::http_server::Settings::new(std::path::PathBuf::from("."))
				.token_lifetime_seconds
				.unwrap()
		});
	for event in events {
		// TODO : security issue : filter by user token !

		if let Some(ref last_event_id) = last_event_id {
			if last_event_id == &event.id {
				id_found = true;
			}
		}

		let allowed_path = if let Some(ref token) = token {
			token
				.is_allowed(
					max_token_lifetime_seconds,
					&actix_web::http::Method::GET,
					&event.path,
				)
				.unwrap_or(false)
		} else {
			false
		};

		if id_found && allowed_path {
			if !reponse_content.is_empty() {
				reponse_content.push('\n');
				reponse_content.push('\n');
			}

			reponse_content += &format!("id: {}\n", event.id);
			reponse_content += &format!(
				"event: {}\n",
				match event.method {
					DbEventMethod::Create => "create",
					DbEventMethod::Update => "update",
					DbEventMethod::Delete => "delete",
				}
			);
			reponse_content += &format!("data: path: {}\n", event.path);
			reponse_content += &format!("data: etag: {}\n", event.etag);
			reponse_content += &format!("data: user: {}\n", event.user);
		}
	}

	let mut res = actix_web::HttpResponse::Ok();
	res.insert_header((actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"));
	res.insert_header((actix_web::http::header::CONTENT_TYPE, "text/event-stream"));
	res.insert_header((actix_web::http::header::CACHE_CONTROL, "no-cache"));

	return res.body(actix_web::web::Bytes::copy_from_slice(
		reponse_content.as_bytes(),
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
