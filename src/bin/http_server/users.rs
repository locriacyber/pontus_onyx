use std::io::{BufRead, Write};

pub fn load_or_create_users(settings: &super::Settings) -> super::Users {
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

				let mut users = super::Users::new();
				if let Err(e) = users.insert(&admin_username, &mut admin_password) {
					println!("\t\t❌ Can not add user `{}` : {}", admin_username, e);
				}

				let dummy = super::UserRight::ManageUsers;
				// this is a little trick to remember to add rights when modified :
				match dummy {
					super::UserRight::ManageUsers => { /* remember to add this right to admin */ }
				}
				let rights = &[super::UserRight::ManageUsers];
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
