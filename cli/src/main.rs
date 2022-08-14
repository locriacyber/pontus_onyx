#![allow(clippy::needless_return)]

use std::convert::From;
use std::sync::{Arc, Mutex};

/*
TODO : continue to :
	https://datatracker.ietf.org/doc/html/draft-dejong-remotestorage-19
		"12. Example wire transcripts"
*/

// TODO : anti brute-force for auth & /public/

// TODO : gracefull panic (like `human_panic` crate but compatible with async) ?

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	println!(
		"{} V{}",
		env!("CARGO_PKG_NAME").to_uppercase(),
		env!("CARGO_PKG_VERSION")
	);
	println!();

	let workspace_path =
		std::path::PathBuf::from(if let Some(workspace_dir) = std::env::args().nth(1) {
			if let Err(err) = std::fs::create_dir_all(workspace_dir.clone()) {
				panic!(
					"Error : can not create workspace {} : {}",
					workspace_dir, err
				);
			}

			workspace_dir
		} else {
			String::from("database")
		});

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

	let localhost = String::from("localhost");

	temp_logger.push(vec![], Some("*CONSOLE_WHITESPACE*"));

	let mut settings_path = workspace_path.clone();
	settings_path.push("settings.toml");
	let settings =
		pontus_onyx::http_server::load_or_create_settings(settings_path.clone(), &mut temp_logger);

	temp_logger.push(vec![], Some("*CONSOLE_WHITESPACE*"));

	let logger =
		pontus_onyx::http_server::load_or_create_logger(&settings, temp_logger, temp_logs_list);

	logger
		.lock()
		.unwrap()
		.push(vec![], Some("*CONSOLE_WHITESPACE*"));

	let users = pontus_onyx::http_server::load_or_create_users(&settings, logger.clone());

	logger
		.lock()
		.unwrap()
		.push(vec![], Some("*CONSOLE_WHITESPACE*"));

	let database = pontus_onyx::http_server::load_or_create_database(&settings, logger.clone());

	logger
		.lock()
		.unwrap()
		.push(vec![], Some("*CONSOLE_WHITESPACE*"));

	let settings = Arc::new(Mutex::new(settings));
	let users = Arc::new(Mutex::new(users));
	let program_state = Arc::new(Mutex::new(pontus_onyx::http_server::ProgramState::default()));

	let oauth_form_tokens: Arc<Mutex<Vec<pontus_onyx::http_server::middlewares::OauthFormToken>>> =
		Arc::new(Mutex::new(vec![]));

	let access_tokens: Arc<Mutex<Vec<pontus_onyx::http_server::AccessBearer>>> =
		Arc::new(Mutex::new(vec![]));

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

	pontus_onyx::http_server::setup_and_run_https_server(
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
		println!("\t⚠ More help : https://github.com/Jimskapt/pontus_onyx/wiki/SSL-cert");
		println!();
	}

	let http_port = settings.lock().unwrap().port;

	logger.lock().unwrap().push(
		vec![
			(String::from("event"), String::from("setup")),
			(String::from("module"), String::from("http")),
			(String::from("level"), String::from("INFO")),
		],
		Some(&format!(
			"API should now listen to http://{}:{http_port}/",
			settings
				.lock()
				.unwrap()
				.domain
				.as_ref()
				.unwrap_or_else(|| &localhost)
		)),
	);

	logger
		.lock()
		.unwrap()
		.push(vec![], Some("*CONSOLE_WHITESPACE*"));

	let handles = {
		let mut handles = String::new();

		let settings = settings.lock().unwrap();
		let users = users.lock().unwrap();

		let mut ports = vec![settings.port];
		if let Some(ref https) = settings.https {
			if program_state.lock().unwrap().https_mode {
				ports = vec![https.port, settings.port];
			}
		}
		let db_users: Vec<String> = users.get_usernames().into_iter().cloned().collect();

		for port in ports {
			for user in &db_users {
				handles += &format!(
					"\n\t- {user}@{}:{port}",
					settings.domain.as_ref().unwrap_or_else(|| &localhost)
				);
			}
		}

		handles
	};

	logger.lock().unwrap().push(
		vec![
			(String::from("event"), String::from("startup")),
			(String::from("level"), String::from("INFO")),
		],
		Some(&format!("Available handles are : {handles}",)),
	);

	logger
		.lock()
		.unwrap()
		.push(vec![], Some("*CONSOLE_WHITESPACE*"));

	let enable_hsts = match &settings.lock().unwrap().https {
		Some(https) => https.enable_hsts,
		None => program_state.lock().unwrap().https_mode,
	};

	let domain = settings
		.lock()
		.unwrap()
		.domain
		.as_ref()
		.unwrap_or_else(|| &localhost)
		.clone();

	let logger_for_server = logger.clone();
	let http_binding = actix_web::HttpServer::new(move || {
		// same code in https module
		actix_web::App::new()
			.wrap(pontus_onyx::http_server::middlewares::Hsts {
				enable: enable_hsts,
			})
			.wrap(pontus_onyx::http_server::middlewares::Auth {
				logger: logger_for_server.clone(),
			})
			.wrap(pontus_onyx::http_server::middlewares::Logger {
				logger: logger_for_server.clone(),
			})
			.configure(pontus_onyx::http_server::configure_server(
				settings.clone(),
				database.clone(),
				access_tokens.clone(),
				oauth_form_tokens.clone(),
				users.clone(),
				program_state.clone(),
				logger_for_server.clone(),
			))
	})
	.bind(format!("{domain}:{http_port}"));

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
