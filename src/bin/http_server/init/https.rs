use std::sync::{Arc, Mutex};

pub fn setup_and_run_https_server(
	settings: Arc<Mutex<super::Settings>>,
	database: Arc<Mutex<pontus_onyx::database::Database>>,
	access_tokens: Arc<Mutex<Vec<crate::http_server::AccessBearer>>>,
	oauth_form_tokens: Arc<Mutex<Vec<crate::http_server::middlewares::OauthFormToken>>>,
	users: Arc<Mutex<crate::http_server::Users>>,
	program_state: Arc<Mutex<crate::ProgramState>>,
	logger: Arc<Mutex<charlie_buffalo::Logger>>,
) {
	let settings_for_setup = settings.lock().unwrap().clone();

	match settings_for_setup.https {
		Some(settings_https) => match std::fs::File::open(&settings_https.keyfile_path) {
			Ok(keyfile_content) => match std::fs::File::open(&settings_https.certfile_path) {
				Ok(cert_content) => {
					let key_file = &mut std::io::BufReader::new(keyfile_content);
					let cert_file = &mut std::io::BufReader::new(cert_content);
					match rustls::internal::pemfile::certs(cert_file) {
						Ok(cert_chain) => {
							match rustls::internal::pemfile::pkcs8_private_keys(key_file) {
								Ok(keys) => {
									let mut server_config =
										rustls::ServerConfig::new(rustls::NoClientAuth::new());

									match keys.get(0) {
										Some(key) => {
											match server_config
												.set_single_cert(cert_chain, key.clone())
											{
												Ok(_) => {
													let enable_hsts = settings_https.enable_hsts;
													let https_port = settings_https.port;

													let program_state_for_server =
														program_state.clone();
													let logger_for_server = logger.clone();
													match actix_web::HttpServer::new(move || {
														actix_web::App::new()
															.wrap(crate::http_server::middlewares::Hsts {
																enable: enable_hsts,
															})
															.wrap(crate::http_server::middlewares::Auth {
																logger: logger_for_server.clone(),
															})
															.wrap(crate::http_server::middlewares::Logger {
																logger: logger_for_server.clone(),
															})
															.configure(
																crate::http_server::configure_server(
																	settings.clone(),
																	database.clone(),
																	access_tokens.clone(),
																	oauth_form_tokens.clone(),
																	users.clone(),
																	program_state_for_server.clone(),
																	logger_for_server.clone()
																)
															)
													})
													.bind_rustls(
														format!("localhost:{}", https_port),
														server_config,
													) {
														Ok(server_bind) => {
															logger.lock().unwrap().push(
																vec![
																	(String::from("event"), String::from("setup")),
																	(String::from("module"), String::from("https")),
																	(String::from("level"), String::from("INFO")),
																],
																Some(&format!("API should now listen to https://localhost:{}/",
																https_port)),
															);

															program_state
																.lock()
																.unwrap()
																.https_mode = true;

															let https_server = server_bind.run();

															std::thread::spawn(move || {
																let mut sys =
																	actix_web::rt::System::new(
																		"https",
																	);
																sys.block_on(https_server)
															});
														}
														Err(e) => {
															logger.lock().unwrap().push(
																vec![
																	(String::from("event"), String::from("setup")),
																	(String::from("module"), String::from("https")),
																	(String::from("level"), String::from("ERROR")),
																],
																Some(&format!("can not set up HTTPS server : {}",
																e)),
															);
														}
													}
												}
												Err(e) => {
													logger.lock().unwrap().push(
														vec![
															(String::from("event"), String::from("setup")),
															(String::from("module"), String::from("https")),
															(String::from("level"), String::from("ERROR")),
														],
														Some(&format!("can not insert certificate in server : {}",
														e)),
													);
												}
											}
										}
										None => {
											logger.lock().unwrap().push(
												vec![
													(String::from("event"), String::from("setup")),
													(String::from("module"), String::from("https")),
													(String::from("level"), String::from("ERROR")),
												],
												Some("no private key found"),
											);
										}
									}
								}
								Err(()) => {
									logger.lock().unwrap().push(
										vec![
											(String::from("event"), String::from("setup")),
											(String::from("module"), String::from("https")),
											(String::from("level"), String::from("ERROR")),
										],
										Some("can not read PKCS8 private key"),
									);
								}
							}
						}
						Err(()) => {
							logger.lock().unwrap().push(
								vec![
									(String::from("event"), String::from("setup")),
									(String::from("module"), String::from("https")),
									(String::from("level"), String::from("ERROR")),
								],
								Some("can not read SSL certificate"),
							);
						}
					}
				}
				Err(e) => {
					logger.lock().unwrap().push(
						vec![
							(String::from("event"), String::from("setup")),
							(String::from("module"), String::from("https")),
							(String::from("level"), String::from("ERROR")),
						],
						Some(&format!(
							"can not open cert file `{}` : {}",
							settings_https.certfile_path, e
						)),
					);
				}
			},
			Err(e) => {
				logger.lock().unwrap().push(
					vec![
						(String::from("event"), String::from("setup")),
						(String::from("module"), String::from("https")),
						(String::from("level"), String::from("ERROR")),
					],
					Some(&format!(
						"can not open key file `{}` : {}",
						settings_https.keyfile_path, e
					)),
				);
			}
		},
		None => {
			logger.lock().unwrap().push(
				vec![
					(String::from("event"), String::from("setup")),
					(String::from("module"), String::from("https")),
					(String::from("level"), String::from("ERROR")),
				],
				Some("no HTTPS settings found"),
			);
		}
	}
}
