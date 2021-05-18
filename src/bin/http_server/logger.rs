use std::sync::{Arc, Mutex};

pub fn load_or_create_logger(
	settings: &super::Settings,
	temp_logger: charlie_buffalo::Logger,
	temp_logs: Arc<Mutex<Vec<charlie_buffalo::Log>>>,
) -> Arc<Mutex<charlie_buffalo::Logger>> {
	let logfile_path = Arc::new(settings.logfile_path.clone());

	if let Some(parents) = std::path::PathBuf::from((*logfile_path).clone()).parent() {
		if let Err(e) = std::fs::create_dir_all(parents) {
			temp_logger.push(
				vec![
					(String::from("event"), String::from("setup")),
					(String::from("module"), String::from("logger")),
					(String::from("level"), String::from("WARNING")),
				],
				Some(&format!(
					"can not creating parent folders of log file : {}",
					e
				)),
			);
		}
	}

	let logfile_path_for_dispatch = logfile_path.clone();

	let new_logger = charlie_buffalo::concurrent_logger_from(charlie_buffalo::Logger::new(
		charlie_buffalo::new_dispatcher(Box::from(move |log: charlie_buffalo::Log| {
			let mut is_whitespace = false;
			if let Some(content) = &log.content {
				if content.trim().to_uppercase() == "*CONSOLE_WHITESPACE*" {
					is_whitespace = true;
				}
			}

			if !is_whitespace {
				let mut new_log = log;

				let attributes: Vec<(String, String)> = vec![charlie_buffalo::Attr::new(
					"time",
					format!("{}", chrono::offset::Local::now()),
				)
				.into()];
				for attribute in attributes {
					new_log.attributes.insert(attribute.0, attribute.1);
				}

				let empty = String::new();
				let event = new_log
					.attributes
					.get("event")
					.unwrap_or(&empty)
					.to_uppercase();
				if event == "HTTP_ACCESS" {
					println!(
						"[{}\t{}] [{}] [{} {}\t{}]\t{}",
						new_log.attributes.get("time").unwrap_or(&empty),
						new_log.attributes.get("client_ip").unwrap_or(&empty),
						new_log.attributes.get("response_code").unwrap_or(&empty),
						new_log
							.attributes
							.get("protocol")
							.unwrap_or(&empty)
							.to_uppercase(),
						new_log.attributes.get("method").unwrap_or(&empty),
						new_log.attributes.get("path").unwrap_or(&empty),
						match new_log.content {
							Some(ref content) => content.clone(),
							None => String::new(),
						}
					);
				} else {
					if let Some(content) = &new_log.content {
						println!(
							"{}{}{}",
							match new_log.attributes.get("module") {
								Some(module) => format!("[{}]\t", module.trim().to_uppercase()),
								None => String::new(),
							},
							match new_log.attributes.get("level").unwrap_or(&empty).as_str() {
								"INFO" => "ℹ ",
								"WARNING" => "⚠ ",
								"ERROR" => "❌ ",
								_ => "",
							},
							content
						);
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
			} else {
				println!();
			}
		})),
		charlie_buffalo::new_dropper(Box::from(|logger: &charlie_buffalo::Logger| {
			logger.push(vec![charlie_buffalo::Flag::from("STOP").into()], None);
		})),
	));

	temp_logger.push(
		vec![
			(String::from("event"), String::from("setup")),
			(String::from("module"), String::from("logger")),
			(String::from("level"), String::from("INFO")),
		],
		Some(&format!("logs will now be saved in `{}`", logfile_path)),
	);

	for log in temp_logs.lock().unwrap().iter() {
		new_logger.lock().unwrap().receive(log.clone());
	}

	new_logger
}
