#![allow(clippy::needless_return)]

use std::convert::From;
use std::sync::{Arc, Mutex};

extern crate pontus_onyx;

#[cfg(feature = "server_bin")]
mod http_server;

/*
TODO : continue to :
	https://datatracker.ietf.org/doc/html/draft-dejong-remotestorage-16
		"12. Example wire transcripts"
*/

// TODO : anti brute-force for auth & /public/

#[cfg(feature = "server_bin")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
	println!(
		"{} V{}",
		env!("CARGO_PKG_NAME").to_uppercase(),
		env!("CARGO_PKG_VERSION")
	);
	println!();

	let workspace_path = std::path::PathBuf::from("database");

	let temp_logs_list = Arc::new(Mutex::new(vec![]));
	let temp_logs_list_for_dispatcher = temp_logs_list.clone();
	let mut temp_logger = charlie_buffalo::Logger::new(
		charlie_buffalo::new_dispatcher(Box::from(move |log: charlie_buffalo::Log| {
			temp_logs_list_for_dispatcher.lock().unwrap().push(log);
		})),
		charlie_buffalo::new_dropper(Box::from(|_: &charlie_buffalo::Logger| {})),
	);

	temp_logger.push(
		vec![
			(String::from("event"), String::from("setup")),
			(String::from("level"), String::from("INFO")),
		],
		Some("setup of the program"),
	);

	temp_logger.push(vec![], Some("*CONSOLE_WHITESPACE*"));

	let mut settings_path = workspace_path.clone();
	settings_path.push("settings.toml");
	let settings = http_server::load_or_create_settings(settings_path.clone(), &mut temp_logger);

	temp_logger.push(vec![], Some("*CONSOLE_WHITESPACE*"));

	let logger = http_server::load_or_create_logger(&settings, temp_logger, temp_logs_list);

	logger
		.lock()
		.unwrap()
		.push(vec![], Some("*CONSOLE_WHITESPACE*"));

	let users = http_server::load_or_create_users(&settings, logger.clone());

	logger
		.lock()
		.unwrap()
		.push(vec![], Some("*CONSOLE_WHITESPACE*"));

	let database = http_server::load_or_create_database(&settings, logger.clone());

	logger
		.lock()
		.unwrap()
		.push(vec![], Some("*CONSOLE_WHITESPACE*"));

	let settings = Arc::new(Mutex::new(settings));
	let users = Arc::new(Mutex::new(users));
	let program_state = Arc::new(Mutex::new(ProgramState::default()));

	let oauth_form_tokens: Arc<Mutex<Vec<http_server::middlewares::OauthFormToken>>> =
		Arc::new(Mutex::new(vec![]));

	let access_tokens: Arc<Mutex<Vec<http_server::AccessBearer>>> = Arc::new(Mutex::new(vec![]));

	logger.lock().unwrap().push(
		vec![
			(String::from("event"), String::from("setup")),
			(String::from("level"), String::from("INFO")),
		],
		Some("starting servers"),
	);

	logger
		.lock()
		.unwrap()
		.push(vec![], Some("*CONSOLE_WHITESPACE*"));

	http_server::setup_and_run_https_server(
		settings.clone(),
		database.clone(),
		access_tokens.clone(),
		oauth_form_tokens.clone(),
		users.clone(),
		program_state.clone(),
		logger.clone(),
	);

	if !program_state.lock().unwrap().https_mode {
		println!();
		println!("\t⚠ Falling back onto HTTP mode");

		println!();
		println!("\t⚠ All data to and from HTTP server can be read and compromised.");
		println!("\t⚠ It should better serve data though HTTPS.");
		println!("\t⚠ You should better fix previous issues and/or get an SSL certificate.");
		println!("\t⚠ More help : https://github.com/Jimskapt/pontus-onyx/wiki/SSL-cert");
		println!();
	}

	let http_post = settings.lock().unwrap().port;

	logger.lock().unwrap().push(
		vec![
			(String::from("event"), String::from("setup")),
			(String::from("module"), String::from("http")),
			(String::from("level"), String::from("INFO")),
		],
		Some(&format!(
			"API should now listen to http://localhost:{}/",
			http_post
		)),
	);

	logger
		.lock()
		.unwrap()
		.push(vec![], Some("*CONSOLE_WHITESPACE*"));

	let enable_hsts = match &settings.lock().unwrap().https {
		Some(https) => https.enable_hsts,
		None => program_state.lock().unwrap().https_mode,
	};

	let logger_for_server = logger.clone();
	let http_binding = actix_web::HttpServer::new(move || {
		// same code in https module
		actix_web::App::new()
			.data(database.clone())
			.data(oauth_form_tokens.clone())
			.data(access_tokens.clone())
			.data(users.clone())
			.data(settings.clone())
			.data(program_state.clone())
			.data(logger_for_server.clone())
			.wrap(http_server::middlewares::Hsts {
				enable: enable_hsts,
			})
			.wrap(http_server::middlewares::Auth {
				logger: logger_for_server.clone(),
			})
			.wrap(http_server::middlewares::Logger {
				logger: logger_for_server.clone(),
			})
			.service(http_server::favicon)
			.service(http_server::get_oauth)
			.service(http_server::post_oauth)
			.service(http_server::webfinger_handle)
			.service(http_server::get_item)
			.service(http_server::head_item)
			.service(http_server::options_item)
			.service(http_server::put_item)
			.service(http_server::delete_item)
			.service(http_server::remotestoragesvg)
			.service(http_server::index)
	})
	.bind(format!("localhost:{}", http_post));

	match http_binding {
		Ok(binding) => binding.run().await,
		Err(e) => {
			logger.lock().unwrap().push(
				vec![
					(String::from("event"), String::from("setup")),
					(String::from("module"), String::from("http")),
					(String::from("level"), String::from("ERROR")),
				],
				Some(&format!("can not set up HTTP server : {}", e)),
			);

			Err(e)
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct ProgramState {
	https_mode: bool,
}

/*
TODO ?
	Servers MAY support Content-Range headers [RANGE] on GET requests,
	but whether or not they do SHOULD be announced both through the
	"http://tools.ietf.org/html/rfc7233" option mentioned below in
	section 10 and through the HTTP 'Accept-Ranges' response header.
*/

/*
TODO :
* 413 if the payload is too large, e.g. when the server has a
		maximum upload size for documents
* 414 if the request URI is too long,
* 416 if Range requests are supported by the server and the Range
		request can not be satisfied,
* 429 if the client makes too frequent requests or is suspected
		of malicious activity,
* 4xx for all malformed requests, e.g. reserved characters in the
		path [URI, section 2.2], as well as for all PUT and DELETE
		requests to folders,
* 507 in case the account is over its storage quota,
*/
/*
TODO :
	All responses MUST carry CORS headers [CORS].
*/
/*
TODO :
	A "http://remotestorage.io/spec/web-authoring" property has been
	proposed with a string value of the fully qualified domain name to
	which web authoring content is published if the server supports web
	authoring as per [AUTHORING]. Note that this extension is a breaking
	extension in the sense that it divides users into "haves", whose
	remoteStorage accounts allow them to author web content, and
	"have-nots", whose remoteStorage account does not support this
	functionality.
*/
/*
TODO :
	The server MAY expire bearer tokens, and MAY require the user to
	register applications as OAuth clients before first use; if no
	client registration is required, the server MUST ignore the value of
	the client_id parameter in favor of relying on the origin of the
	redirect_uri parameter for unique client identification. See section
	4 of [ORIGIN] for computing the origin.
*/
/*
TODO :
	11. Storage-first bearer token issuance

	To request that the application connects to the user account
	<account> ' ' <host>, providers MAY redirect to applications with a
	'remotestorage' field in the URL fragment, with the user account as
	value.

	The appplication MUST make sure this request is intended by the
	user. It SHOULD ask for confirmation from the user whether they want
	to connect to the given provider account. After confirmation, it
	SHOULD connect to the given provider account, as defined in Section
	10.

	If the 'remotestorage' field exists in the URL fragment, the
	application SHOULD ignore any other parameters such as
	'access_token' or 'state'
*/

#[cfg(not(feature = "server_bin"))]
fn main() {
	eprintln!(r#"WARNING : please build this binary at least with `--features server_bin`"#);
}
