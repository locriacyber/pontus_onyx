#![allow(clippy::needless_return)]

use std::convert::From;
use std::io::{BufRead, Write};
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
	std::env::set_var(
		"RUST_LOG",
		"actix_example=debug,actix_web=debug,actix_http=debug,actix_service=debug",
	);
	env_logger::init();

	println!(
		"{} V{}",
		env!("CARGO_PKG_NAME").to_uppercase(),
		env!("CARGO_PKG_VERSION")
	);
	println!();

	println!("📢 Set up ...");
	println!();

	let workspace_path = std::path::PathBuf::from("database");

	let mut settings_path = workspace_path.clone();
	settings_path.push("settings.toml");

	let settings = match std::fs::read(&settings_path) {
		Ok(bytes) => match toml::from_slice(&bytes) {
			Ok(settings) => settings,
			Err(e) => {
				println!("\t⚠ Can not parse settings file : {}", e);
				println!("\t✔ Falling back to default settings.");

				Settings::default()
			}
		},
		Err(e) => {
			println!("\t⚠ Can not read settings file : {}", e);

			let result = Settings::default();

			if e.kind() == std::io::ErrorKind::NotFound {
				if let Some(parent) = settings_path.parent() {
					if let Err(e) = std::fs::create_dir_all(parent) {
						println!(
							"\t\t❌ Can not creating parent folders of settings file : {}",
							e
						);
					}
				}

				match std::fs::write(settings_path, toml::to_vec(&result).unwrap()) {
					Ok(_) => {
						println!("\t\t✔ Creating default settings file.");
					}
					Err(e) => {
						println!("\t\t❌ Can not creating default settings file : {}", e);
					}
				}
			}

			println!("\t\t✔ Falling back to default settings.");
			println!();

			result
		}
	};
	let settings_for_main = settings.clone();
	let settings = Arc::new(Mutex::new(settings));

	let users_path = std::path::PathBuf::from(settings.lock().unwrap().userfile_path.clone());

	let users = {
		let userlist = match std::fs::read(&settings.lock().unwrap().userfile_path) {
			Ok(bytes) => match bincode::deserialize::<http_server::Users>(&bytes) {
				Ok(users) => Ok(users),
				Err(e) => Err(format!("can not parse users file : {}", e)),
			},
			Err(e) => Err(format!("can not read users file : {}", e)),
		};

		match userlist {
			Ok(userlist) => userlist,
			Err(_) => {
				println!(
					"\t⚠ Users list not found in {}.",
					users_path.to_str().unwrap_or_default()
				);

				println!("\t\t📢 Attempting to create a new one.");

				let mut admin_username = String::new();
				let mut input_is_correct = false;
				while !input_is_correct {
					print!("\t\tPlease type new admin username (or abort all with Ctrl + C) : ");
					std::io::stdout().flush().ok();
					if let Err(e) = std::io::stdin().lock().read_line(&mut admin_username) {
						println!("\t\t\t❌ Can not read your input : {}", e);
					}

					admin_username = admin_username.trim().to_lowercase();

					if EASY_TO_GUESS_USERS.contains(&admin_username.as_str()) {
						println!("\t\t\t❌ This username is too easy to guess");
						admin_username = String::new();
					} else {
						input_is_correct = true;
					}
				}

				let mut admin_password = String::new();
				let mut input_is_correct = false;
				while !input_is_correct {
					admin_password = String::new();

					match rpassword::read_password_from_tty(Some(&format!("\t\tPlease type new password for `{}` (it do not shows for security purposes) : ", admin_username))) {
						Ok(password1) => {
							match rpassword::read_password_from_tty(Some("\t\tPlease type again this password to confirm it : ")) {
								Ok(password2) => {
									if password1 == password2 {
										if password2.trim().chars().count() < 6 {
											println!("\t\t\t❌ This password need at least 6 characters");
										} else {
											input_is_correct = true;
											admin_password = String::from(password2.trim());
										}
									} else {
										println!("\t\t❌ Passwords does not match, please try again");
									}
								},
								Err(e) => println!("\t\t\t❌ Can not read your input : {}", e),
							}
						},
						Err(e) => println!("\t\t\t❌ Can not read your input : {}", e),
					}
				}

				let mut users = http_server::Users::new();
				if let Err(e) = users.insert(&admin_username, &mut admin_password) {
					println!("\t\t❌ Can not add user `{}` : {}", admin_username, e);
				}

				let dummy = http_server::UserRight::ManageUsers;
				// this is a little trick to remember to add rights when modified :
				match dummy {
					http_server::UserRight::ManageUsers => { /* remember to add this right to admin */
					}
				}
				let rights = &[http_server::UserRight::ManageUsers];
				for right in rights {
					if let Err(e) = users.add_right(&admin_username, right.clone()) {
						println!(
							"\t\t❌ Can not add `{}` right to `{}` : {}",
							right, admin_username, e
						);
					}
				}

				if let Some(parent) = users_path.parent() {
					if let Err(e) = std::fs::create_dir_all(parent) {
						println!("\t\t❌ Can not create parent folders of user file : {}", e);
					}
				}
				if let Err(e) = std::fs::write(users_path, bincode::serialize(&users).unwrap()) {
					println!("\t\t❌ Can not create user file : {}", e);
				}

				println!();
				println!(
					"\t\t✔ New users list successfully created, with administrator `{}`.",
					&admin_username
				);
				println!();

				users
			}
		}
	};
	let users = Arc::new(Mutex::new(users));

	let db_path = std::path::PathBuf::from(settings.lock().unwrap().data_path.clone());
	let data_source = pontus_onyx::database::DataSource::File(db_path.clone());

	let (database, change_receiver) = match pontus_onyx::Database::new(data_source.clone()) {
		Ok(e) => e,
		Err(pontus_onyx::database::ErrorNewDatabase::FileDoesNotExists) => {
			println!(
				"\t⚠ Database does not exists in {}.",
				db_path.to_str().unwrap_or_default()
			);

			println!();
			println!("\t\t✔ New empty database created.");
			println!();

			pontus_onyx::Database::new(pontus_onyx::database::DataSource::Memory(
				pontus_onyx::Item::new_folder(vec![]),
			))
			.unwrap()
		}
		Err(e) => {
			panic!("{}", e);
		}
	};
	let database = Arc::new(Mutex::new(database));

	let database_for_save = database.clone();
	std::thread::spawn(move || loop {
		match change_receiver.recv() {
			Ok(event) => database_for_save.lock().unwrap().save_event_into(
				event,
				pontus_onyx::database::DataSource::File(db_path.clone()),
			),
			Err(e) => panic!("{}", e),
		}
	});

	let program_state = Arc::new(Mutex::new(ProgramState::default()));

	let oauth_form_tokens: Arc<Mutex<Vec<http_server::middlewares::OauthFormToken>>> =
		Arc::new(Mutex::new(vec![]));

	// TODO : only for rapid test purposes, delete it for release of the product !
	let access_tokens: Arc<Mutex<Vec<http_server::AccessBearer>>> =
		Arc::new(Mutex::new(/*if cfg!(debug_assertions) {
			let debug_bearer = http_server::AccessBearer::new(
				vec![http_server::Scope {
					module: String::from("*"),
					right_type: http_server::ScopeRightType::ReadWrite,
				}],
				"TODO",
				users.lock().unwrap().get_usernames().get(0).unwrap(),
			);

			println!("🧨 DEBUG BEARER :");
			println!("Bearer {}", debug_bearer.get_name());
			println!();

			vec![debug_bearer]
		} else {
			vec![]
		}*/
		vec![]));

	// TODO : save access_tokens in file ?

	let logfile_path = Arc::new(settings.lock().unwrap().logfile_path.clone());

	if let Some(parents) = std::path::PathBuf::from((*logfile_path).clone()).parent() {
		if let Err(e) = std::fs::create_dir_all(parents) {
			println!("\t\t❌ Can not creating parent folders of log file : {}", e);
		}
	}

	let logfile_path_for_dispatch = logfile_path.clone();
	let logger = charlie_buffalo::concurrent_logger_from(charlie_buffalo::Logger::new(
		charlie_buffalo::new_dispatcher(Box::from(move |log: charlie_buffalo::Log| {
			let mut new_log = log;

			let attributes: Vec<(String, String)> = vec![charlie_buffalo::Attr::new(
				"time",
				format!("{}", chrono::offset::Local::now()),
			)
			.into()];
			for attribute in attributes {
				new_log.attributes.insert(attribute.0, attribute.1);
			}

			match new_log.attributes.get("level") {
				Some(level) => {
					if level == LogLevel::PANIC || level == LogLevel::ERROR {
						eprintln!("{}", &new_log);
					} else {
						println!("{}", &new_log);
					}
				}
				_ => {
					println!("{}", &new_log);
				}
			}

			let mut result: Vec<charlie_buffalo::Log> = rmp_serde::decode::from_slice(
				std::fs::read((*logfile_path_for_dispatch).clone())
					.unwrap_or_default()
					.as_slice(),
			)
			.unwrap_or_default();
			result.push(new_log);
			std::fs::write(
				(*logfile_path_for_dispatch).clone(),
				rmp_serde::encode::to_vec(&result).unwrap(),
			)
			.ok();
		})),
		charlie_buffalo::new_dropper(Box::from(|logger: &charlie_buffalo::Logger| {
			logger.push(vec![charlie_buffalo::Flag::from("STOP").into()], None);
		})),
	));

	/*
	charlie_buffalo::push(
		&logger,
		vec![
			LogLevel::DEBUG.into(),
			charlie_buffalo::Attr::new("code", format!("{}:{}", file!(), line!())).into(),
		],
		Some("logger created"),
	);
	*/

	println!("📢 Attempting to start server ...");
	println!();

	match std::fs::File::open(&settings_for_main.https.keyfile_path) {
		Ok(keyfile_content) => match std::fs::File::open(&settings_for_main.https.certfile_path) {
			Ok(cert_content) => {
				let key_file = &mut std::io::BufReader::new(keyfile_content);
				let cert_file = &mut std::io::BufReader::new(cert_content);
				match rustls::internal::pemfile::certs(cert_file) {
					Ok(cert_chain) => {
						match rustls::internal::pemfile::pkcs8_private_keys(key_file) {
							Ok(mut keys) => {
								let mut server_config =
									rustls::ServerConfig::new(rustls::NoClientAuth::new());

								match server_config.set_single_cert(cert_chain, keys.remove(0)) {
									Ok(_) => {
										let database_for_server = database.clone();
										let oauth_form_tokens_for_server =
											oauth_form_tokens.clone();
										let access_tokens_for_server = access_tokens.clone();
										let users_for_server = users.clone();
										let settings_for_server = settings.clone();
										let program_state_for_server = program_state.clone();
										let logger_for_server = logger.clone();

										let enable_hsts = settings_for_main.https.enable_hsts;

										match actix_web::HttpServer::new(move || {
											actix_web::App::new()
												.data(database_for_server.clone())
												.data(oauth_form_tokens_for_server.clone())
												.data(access_tokens_for_server.clone())
												.data(users_for_server.clone())
												.data(settings_for_server.clone())
												.data(program_state_for_server.clone())
												.data(logger_for_server.clone())
												.wrap(http_server::middlewares::Hsts {
													enable: enable_hsts,
												})
												.wrap(http_server::middlewares::Auth {})
												.wrap(actix_web::middleware::Logger::default())
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
										.bind_rustls(
											format!("localhost:{}", settings_for_main.https.port),
											server_config,
										) {
											Ok(server_bind) => {
												println!(
													"\t✔ API should now listen to https://localhost:{}/",
													settings_for_main.https.port
												);

												{
													program_state.lock().unwrap().https_mode = true;
												}

												let https_server = server_bind.run();

												std::thread::spawn(move || {
													let mut sys =
														actix_web::rt::System::new("https");
													sys.block_on(https_server)
												});
											}
											Err(e) => {
												println!(
													"\t❌ Can not set up HTTPS server : {}",
													e
												);
											}
										}
									}
									Err(e) => {
										println!(
											"\t⚠ HTTPS : can add certificate in server : {}",
											e
										);
									}
								}
							}
							Err(()) => {
								println!("\t⚠ HTTPS : can not read PKCS8 private key");
							}
						}
					}
					Err(()) => {
						println!("\t⚠ HTTPS : can not read SSL certificate");
					}
				}
			}
			Err(e) => {
				println!(
					"\t⚠ HTTPS : can not open cert file `{}` : {}",
					settings_for_main.https.certfile_path, e
				);
			}
		},
		Err(e) => {
			println!(
				"\t⚠ HTTPS : can not open key file `{}` : {}",
				settings_for_main.https.keyfile_path, e
			);
		}
	}

	if !program_state.lock().unwrap().https_mode {
		println!();
		println!("\t📢 Falling back onto HTTP mode");

		println!();
		println!("\t⚠ All data to and from this HTTP server can be read and compromised.");
		println!("\tIt should better serve data though HTTPS.");
		println!("\tYou should better fix previous issues and/or get an SSL certificate.");
		println!("\tMore help : https://github.com/Jimskapt/pontus-onyx/wiki/SSL-cert");
		println!();
	}

	println!(
		"\t✔ API should now listen to http://localhost:{}/",
		settings_for_main.port
	);
	println!();

	let database_for_server = database.clone();
	let oauth_form_tokens_for_server = oauth_form_tokens.clone();
	let access_tokens_for_server = access_tokens.clone();
	let users_for_server = users.clone();
	let settings_for_server = settings.clone();
	let program_state_for_server = program_state.clone();
	let logger_for_server = logger.clone();

	actix_web::HttpServer::new(move || {
		actix_web::App::new()
			.data(database_for_server.clone())
			.data(oauth_form_tokens_for_server.clone())
			.data(access_tokens_for_server.clone())
			.data(users_for_server.clone())
			.data(settings_for_server.clone())
			.data(program_state_for_server.clone())
			.data(logger_for_server.clone())
			.wrap(http_server::middlewares::Auth {})
			.wrap(actix_web::middleware::Logger::default())
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
	.bind(format!("localhost:{}", settings_for_main.port))
	.expect("❌ Can not set up HTTP server, abort launching.")
	.run()
	.await
}

const EASY_TO_GUESS_USERS: &[&str] = &[
	"",
	"admin",
	"administrator",
	"root",
	"main",
	"user",
	"username",
];

#[derive(serde::Serialize)]
enum LogLevel {
	DEBUG,
	INFO,
	WARN,
	ERROR,
	PANIC,
}
impl charlie_buffalo::ValueAsString for LogLevel {
	fn as_string(&self) -> String {
		format!(
			"{}",
			match self {
				LogLevel::DEBUG => 10,
				LogLevel::INFO => 20,
				LogLevel::WARN => 30,
				LogLevel::ERROR => 40,
				LogLevel::PANIC => 50,
			}
		)
	}
}
impl std::convert::From<LogLevel> for (String, String) {
	fn from(input: LogLevel) -> Self {
		return (
			String::from("level"),
			charlie_buffalo::ValueAsString::as_string(&input),
		);
	}
}
impl std::cmp::PartialEq<LogLevel> for &String {
	fn eq(&self, other: &LogLevel) -> bool {
		*self == &charlie_buffalo::ValueAsString::as_string(other)
	}
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Settings {
	port: usize,
	admin_email: String,
	token_lifetime_seconds: u64,
	logfile_path: String,
	userfile_path: String,
	data_path: String,
	https: SettingsHTTPS,
}
impl Default for Settings {
	fn default() -> Self {
		Self {
			port: 7541,
			admin_email: String::from(""),
			token_lifetime_seconds: 60 * 60,
			logfile_path: String::from("database/logs.msgpack"),
			userfile_path: String::from("database/users.bin"),
			data_path: String::from("database/data"),
			https: SettingsHTTPS::default(),
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct SettingsHTTPS {
	port: usize,
	keyfile_path: String,
	certfile_path: String,
	enable_hsts: bool,
}
impl Default for SettingsHTTPS {
	fn default() -> Self {
		Self {
			port: 7542,
			keyfile_path: String::new(),
			certfile_path: String::new(),
			enable_hsts: true,
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
* 401 for all requests that require a valid bearer token and
		where no valid one was sent (see also [BEARER, section
		3.1]),
* 403 for all requests that have insufficient scope, e.g.
		accessing a <module> for which no scope was obtained, or
		accessing data outside the user's <storage_root>,
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
