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

	println!("📢 Read settings ...");
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
					std::fs::create_dir_all(parent);
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

	let mut users_path = workspace_path.clone();
	users_path.push("users.bin");

	let users = {
		let userlist = match std::fs::read(&users_path) {
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
					print!("\t\t❓ Please type new admin username (or abort all with Ctrl + C) : ");
					std::io::stdout().flush();
					std::io::stdin().lock().read_line(&mut admin_username);

					admin_username = admin_username.trim().to_lowercase();

					if EASY_TO_GUESS_USERS.contains(&admin_username.as_str()) {
						println!("\t\t\t❌ Error : this username is too easy to guess.");
						admin_username = String::new();
					} else {
						input_is_correct = true;
					}
				}

				let mut admin_password = String::new();
				let mut input_is_correct = false;
				while !input_is_correct {
					admin_password = rpassword::read_password_from_tty(Some(&format!("\t\t❓ Please type new password for `{}` (it do not shows for security purposes) : ", admin_username))).unwrap();
					admin_password = admin_password.trim().to_lowercase();

					if admin_password.chars().count() < 6 {
						println!("\t\t\t❌ Error : this password need at least 6 characters.");
						admin_password = String::new();
					} else {
						input_is_correct = true;
					}
				}

				let mut users = http_server::Users::new();
				users.insert(&admin_username, &mut admin_password);

				let dummy = http_server::UserRight::ManageUsers;
				// this is a little trick to remember to add rights when modified :
				match dummy {
					http_server::UserRight::ManageUsers => { /* remember to add this right to admin */
					}
				}
				users.add_right(&admin_username, http_server::UserRight::ManageUsers);

				if let Some(parent) = users_path.parent() {
					std::fs::create_dir_all(parent);
				}
				std::fs::write(users_path, bincode::serialize(&users).unwrap());

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

	let mut db_path = workspace_path.clone();
	db_path.push("data");
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

	let oauth_form_tokens: Arc<Mutex<Vec<http_server::middlewares::OauthFormToken>>> =
		Arc::new(Mutex::new(vec![]));

	// TODO : only for rapid test purposes, delete it for release of the product !
	let access_tokens: Arc<Mutex<Vec<http_server::AccessBearer>>> =
		Arc::new(Mutex::new(if cfg!(debug_assertions) {
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
		}));

	println!("📢 Attempting to start server ...");
	println!();

	let mut https_mode = false;

	match std::fs::File::open(&settings.https.keyfile_path) {
		Ok(keyfile_content) => match std::fs::File::open(&settings.https.certfile_path) {
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

										let enable_hsts = settings.https.enable_htsts;

										match actix_web::HttpServer::new(move || {
											actix_web::App::new()
												.data(database_for_server.clone())
												.data(oauth_form_tokens_for_server.clone())
												.data(access_tokens_for_server.clone())
												.data(users_for_server.clone())
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
											format!("localhost:{}", settings.port),
											server_config,
										) {
											Ok(server_bind) => {
												println!(
													"\t✔ API should now listen to https://localhost:{}/",
													settings.port
												);
												println!();

												https_mode = true;

												server_bind.run().await.unwrap();
											}
											Err(e) => {
												println!("\t❌ Can not set up HTTP server : {}", e);
											}
										}
									}
									Err(e) => {
										println!("\t⚠ Can add certificate in server : {}", e);
									}
								}
							}
							Err(()) => {
								println!("\t⚠ Can not read PKCS8 private key");
							}
						}
					}
					Err(()) => {
						println!("\t⚠ Can not read SSL certificate");
					}
				}
			}
			Err(e) => {
				println!(
					"\t⚠ Can not open cert file `{}` : {}",
					settings.https.certfile_path, e
				);
			}
		},
		Err(e) => {
			println!(
				"\t⚠ Can not open key file `{}` : {}",
				settings.https.keyfile_path, e
			);
		}
	}

	if !https_mode {
		println!();
		println!("\t📢 Falling back onto HTTP mode");

		println!();
		println!("\t⚠ All data to and from this HTTP server can be read and compromised.");
		println!("\tIt should better serve data though HTTPS.");
		println!("\tYou should better fix previous issues and/or get an SSL certificate.");
		println!("\tMore help : https://github.com/Jimskapt/pontus-onyx/wiki/SSL-cert");
		println!();

		println!(
			"\t✔ API should now listen to http://localhost:{}/",
			settings.port
		);
		println!();

		let database_for_server = database.clone();
		let oauth_form_tokens_for_server = oauth_form_tokens.clone();
		let access_tokens_for_server = access_tokens.clone();
		let users_for_server = users.clone();

		actix_web::HttpServer::new(move || {
			actix_web::App::new()
				.data(database_for_server.clone())
				.data(oauth_form_tokens_for_server.clone())
				.data(access_tokens_for_server.clone())
				.data(users_for_server.clone())
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
		.bind(format!("localhost:{}", settings.port))
		.expect("❌ Can not set up HTTP server, abort launching.")
		.run()
		.await
		.unwrap();
	}

	Ok(())
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Settings {
	port: usize,
	admin_email: String,
	https: SettingsHTTPS,
}
impl Default for Settings {
	fn default() -> Self {
		Self {
			port: 7541,
			admin_email: String::from(""),
			https: SettingsHTTPS::default(),
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct SettingsHTTPS {
	keyfile_path: String,
	certfile_path: String,
	enable_htsts: bool,
}
impl Default for SettingsHTTPS {
	fn default() -> Self {
		Self {
			keyfile_path: String::new(),
			certfile_path: String::new(),
			enable_htsts: true,
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
