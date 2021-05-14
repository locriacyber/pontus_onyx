#![allow(clippy::needless_return)]

use std::convert::From;
use std::io::{BufRead, Write};

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
	let users = std::sync::Arc::new(std::sync::Mutex::new(users));

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
	let database = std::sync::Arc::new(std::sync::Mutex::new(database));

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

	let oauth_form_tokens: std::sync::Arc<std::sync::Mutex<Vec<http_server::OauthFormToken>>> =
		std::sync::Arc::new(std::sync::Mutex::new(vec![]));

	// TODO : only for rapid test purposes, delete it for release of the product !
	let access_tokens: std::sync::Arc<std::sync::Mutex<Vec<http_server::AccessBearer>>> =
		std::sync::Arc::new(std::sync::Mutex::new(if cfg!(debug_assertions) {
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

	let server = actix_web::HttpServer::new(move || {
		actix_web::App::new()
			.data(database.clone())
			.data(oauth_form_tokens.clone())
			.data(access_tokens.clone())
			.data(users.clone())
			.wrap(http_server::Auth {})
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
	});

	/*
	match openssl::ssl::SslAcceptor::mozilla_intermediate(
		openssl::ssl::SslMethod::tls()
	) {
		Ok(mut ssl_builder) => {
			match ssl_builder.set_private_key_file(settings.keyfile_path, openssl::ssl::SslFiletype::PEM) {
				Ok(_) => {
					match ssl_builder.set_certificate_chain_file(settings.certfile_path) {
						Ok(_) => {
							match server
							.bind_openssl(format!("localhost:{}", settings.port), ssl_builder) {
								Ok(server) => {
									println!("✔ API should now listen to https://localhost:{}/", settings.port);
									server.run();
								}
								Err(e) => {
									println!("⚠ Can not serve with OpenSSL : {}", e);
								}
							}
						},
						Err(e) => {
							println!("⚠ Error while using cert file `{}` with OpenSSL : {}", settings.certfile_path, e);
						}
					}
				},
				Err(e) => {
					println!("⚠ Error while using key file `{}` with OpenSSL : {}", settings.keyfile_path, e);
				}
			}
		},
		Err(e) => {
			println!("⚠ Error while creating OpenSSL utility : {}", e);
		}
	}
	*/

	/*
	let cert_content = std::fs::read("device.key").unwrap();
	dbg!(&cert_content.is_empty());

	let codec = rustls::internal::msgs::codec::Codec::read_bytes(&cert_content);
	dbg!(&codec);

	let mut cert_store = rustls::RootCertStore::empty();
	cert_store.add(&codec.unwrap()).unwrap();
	*/

	/*
	let cert_content = std::fs::read("cert.pem").unwrap();
	let mut cert_content = cert_content.as_slice();

	let mut cert_store = rustls::RootCertStore::empty();
	cert_store.add_pem_file(&mut cert_content).unwrap();

	let config = rustls::ServerConfig::new(
		// AllowAnyAuthenticatedClient
		rustls::AllowAnyAnonymousOrAuthenticatedClient::new(
			cert_store
		)
	);

	match server
		.bind_rustls(format!("localhost:{}", settings.port), config) {
		Ok(server) => {
			println!("✔ API should now listen to https://localhost:{}/", settings.port);
			server.run().await
		}
		Err(e) => {
			println!("⚠ Can not serve with rustls : {}", e);
			Ok(())
		}
	}
	*/

	println!("\t❌ HTTPS not ready, yet.");
	println!();

	println!("\t⚠ Falling back onto HTTP mode");

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

	server
		.bind(format!("localhost:{}", settings.port))? // TODO : HTTPS
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

#[derive(serde::Serialize, serde::Deserialize)]
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

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct SettingsHTTPS {
	keyfile_path: String,
	certfile_path: String,
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
