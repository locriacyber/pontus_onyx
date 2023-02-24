use std::io::{BufRead, Write};

pub fn load_or_create_users(
	settings: &super::Settings,
	logger: std::sync::Arc<std::sync::Mutex<charlie_buffalo::Logger>>,
) -> crate::http_server::Users {
	let users_path = settings.userfile_path();

	let users = {
		let userlist = match std::fs::read(&users_path) {
			Ok(bytes) => match bincode::deserialize::<crate::http_server::Users>(&bytes) {
				Ok(users) => Ok(users),
				Err(e) => Err(format!("can not parse users file : {}", e)),
			},
			Err(e) => Err(format!("can not read users file : {}", e)),
		};

		match userlist {
			Ok(userlist) => {
				logger.lock().unwrap().push(
					vec![
						(String::from("event"), String::from("setup")),
						(String::from("module"), String::from("users_list")),
						(String::from("level"), String::from("INFO")),
					],
					Some(&format!(
						"users successfully loaded from `{}`",
						&users_path.to_string_lossy(),
					)),
				);

				userlist
			}
			Err(e) => {
				logger.lock().unwrap().push(
					vec![
						(String::from("event"), String::from("setup")),
						(String::from("module"), String::from("users_list")),
						(String::from("level"), String::from("WARNING")),
					],
					Some(&format!(
						"users list not found in `{}` : {e}",
						dunce::canonicalize(&users_path)
							.unwrap_or_else(|_| users_path.clone())
							.display()
					)),
				);

				logger.lock().unwrap().push(
					vec![
						(String::from("event"), String::from("setup")),
						(String::from("module"), String::from("users_list")),
						(String::from("level"), String::from("INFO")),
					],
					Some("trying to create a new users list"),
				);

				let mut admin_username = String::new();
				let mut input_is_correct = false;
				while !input_is_correct {
					print!("\t✏️ Please type new admin username (or abort all with Ctrl + C) : ");
					std::io::stdout().flush().ok();
					if let Err(e) = std::io::stdin().lock().read_line(&mut admin_username) {
						println!("\t\t❌ Can not read your input : {}", e);
					}

					admin_username = admin_username.trim().to_lowercase();
					input_is_correct = true;
				}

				let mut admin_password = String::new();
				let mut input_is_correct = false;
				while !input_is_correct {
					admin_password = String::new();

					print!("\t✏️ Please type new password for `{}` (it do not shows for security purposes) : ", admin_username);
					std::io::stdout().flush().ok();

					match rpassword::read_password() {
						Ok(password1) => {
							print!("\t✏️ Please type again this password to confirm it : ");
							std::io::stdout().flush().ok();

							match rpassword::read_password() {
								Ok(password2) => {
									if password1 == password2 {
										admin_password = password2;
										input_is_correct = true;
									} else {
										println!("\t❌ Passwords does not match, please try again");
									}
								}
								Err(e) => println!("\t\t❌ Can not read your input : {}", e),
							}
						}
						Err(e) => println!("\t\t❌ Can not read your input : {}", e),
					}
				}

				let mut users = crate::http_server::Users::new();
				if let Err(e) = users.insert(&admin_username, &mut admin_password) {
					logger.lock().unwrap().push(
						vec![
							(String::from("event"), String::from("setup")),
							(String::from("module"), String::from("users_list")),
							(String::from("level"), String::from("ERROR")),
						],
						Some(&format!("can not add user `{}` : {}", admin_username, e)),
					);

					panic!();
				}

				let dummy = crate::http_server::UserRight::ManageUsers;
				// this is a little trick to remember to add rights when modified :
				match dummy {
					crate::http_server::UserRight::ManageServerSettings => {
						/* remember to add this right to admin */
					}
					crate::http_server::UserRight::ManageUsers => {
						/* remember to add this right to admin */
					}
					crate::http_server::UserRight::ManageApplications => {
						/* remember to add this right to admin */
					}
				}
				let rights = &[
					crate::http_server::UserRight::ManageServerSettings,
					crate::http_server::UserRight::ManageUsers,
					crate::http_server::UserRight::ManageApplications,
				];
				for right in rights {
					if let Err(e) = users.add_right(&admin_username, right.clone()) {
						logger.lock().unwrap().push(
							vec![
								(String::from("event"), String::from("setup")),
								(String::from("module"), String::from("users_list")),
								(String::from("level"), String::from("WARNING")),
							],
							Some(&format!(
								"can not add `{}` right to `{}` : {}",
								right, admin_username, e
							)),
						);
					}
				}

				if let Some(parent) = users_path.parent() {
					if let Err(e) = std::fs::create_dir_all(parent) {
						logger.lock().unwrap().push(
							vec![
								(String::from("event"), String::from("setup")),
								(String::from("module"), String::from("users_list")),
								(String::from("level"), String::from("WARNING")),
							],
							Some(&format!(
								"can not create parent folders of user file : {}",
								e
							)),
						);
					}
				}
				if let Err(e) = std::fs::write(users_path, bincode::serialize(&users).unwrap()) {
					logger.lock().unwrap().push(
						vec![
							(String::from("event"), String::from("setup")),
							(String::from("module"), String::from("users_list")),
							(String::from("level"), String::from("WARNING")),
						],
						Some(&format!("can not create user file : {}", e)),
					);
				}

				logger.lock().unwrap().push(
					vec![
						(String::from("event"), String::from("setup")),
						(String::from("module"), String::from("users_list")),
						(String::from("level"), String::from("INFO")),
					],
					Some(&format!(
						"new users list successfully created, with administrator `{}`",
						&admin_username
					)),
				);

				users
			}
		}
	};

	users
}
