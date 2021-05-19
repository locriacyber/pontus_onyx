use std::io::{BufRead, Write};

pub fn load_or_create_users(
	settings: &super::Settings,
	logger: std::sync::Arc<std::sync::Mutex<charlie_buffalo::Logger>>,
) -> super::Users {
	let users_path = std::path::PathBuf::from(settings.userfile_path.clone());

	let users = {
		let userlist = match std::fs::read(&settings.userfile_path) {
			Ok(bytes) => match bincode::deserialize::<super::Users>(&bytes) {
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
						"users list not found in `{}` : {}",
						&users_path.to_string_lossy(),
						e
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
					print!("\tPlease type new admin username (or abort all with Ctrl + C) : ");
					std::io::stdout().flush().ok();
					if let Err(e) = std::io::stdin().lock().read_line(&mut admin_username) {
						println!("\t\t❌ Can not read your input : {}", e);
					}

					admin_username = admin_username.trim().to_lowercase();

					if EASY_TO_GUESS_USERS.contains(&admin_username.as_str()) {
						println!("\t❌ This username is too easy to guess");
						admin_username = String::new();
					} else {
						input_is_correct = true;
					}
				}

				let mut admin_password = String::new();
				let mut input_is_correct = false;
				while !input_is_correct {
					admin_password = String::new();

					match rpassword::read_password_from_tty(Some(&format!("\tPlease type new password for `{}` (it do not shows for security purposes) : ", admin_username))) {
						Ok(password1) => {
							match rpassword::read_password_from_tty(Some("\tPlease type again this password to confirm it : ")) {
								Ok(password2) => {
									if password1 == password2 {
										if password2.trim().chars().count() < 6 {
											println!("\t❌ This password need at least 6 characters");
										} else {
											input_is_correct = true;
											admin_password = String::from(password2.trim());
										}
									} else {
										println!("\t❌ Passwords does not match, please try again");
									}
								},
								Err(e) => println!("\t\t❌ Can not read your input : {}", e),
							}
						},
						Err(e) => println!("\t\t❌ Can not read your input : {}", e),
					}
				}

				let mut users = super::Users::new();
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

				let dummy = super::UserRight::ManageUsers;
				// this is a little trick to remember to add rights when modified :
				match dummy {
					super::UserRight::ManageUsers => { /* remember to add this right to admin */ }
				}
				let rights = &[super::UserRight::ManageUsers];
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

const EASY_TO_GUESS_USERS: &[&str] = &[
	"",
	"admin",
	"administrator",
	"root",
	"main",
	"user",
	"username",
];
