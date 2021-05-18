use std::sync::{Arc, Mutex};

pub fn setup_and_run_https_server(
	settings: Arc<Mutex<super::Settings>>,
	database: Arc<Mutex<pontus_onyx::Database>>,
	access_tokens: Arc<Mutex<Vec<super::AccessBearer>>>,
	oauth_form_tokens: Arc<Mutex<Vec<super::middlewares::OauthFormToken>>>,
	users: Arc<Mutex<super::Users>>,
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
															.data(database.clone())
															.data(oauth_form_tokens.clone())
															.data(access_tokens.clone())
															.data(users.clone())
															.data(settings.clone())
															.data(program_state_for_server.clone())
															.data(logger_for_server.clone())
															.wrap(super::middlewares::Hsts {
																enable: enable_hsts,
															})
															.wrap(super::middlewares::Auth {})
															.wrap(super::middlewares::Logger {
																logger: logger_for_server.clone(),
															})
															.service(super::favicon)
															.service(super::get_oauth)
															.service(super::post_oauth)
															.service(super::webfinger_handle)
															.service(super::get_item)
															.service(super::head_item)
															.service(super::options_item)
															.service(super::put_item)
															.service(super::delete_item)
															.service(super::remotestoragesvg)
															.service(super::index)
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
