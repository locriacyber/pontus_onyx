use std::sync::{Arc, Mutex};

pub fn load_or_create_logger(
	settings: Arc<Mutex<super::Settings>>,
) -> Arc<Mutex<charlie_buffalo::Logger>> {
	let logfile_path = Arc::new(settings.lock().unwrap().logfile_path.clone());

	if let Some(parents) = std::path::PathBuf::from((*logfile_path).clone()).parent() {
		if let Err(e) = std::fs::create_dir_all(parents) {
			println!("\t\t❌ Can not creating parent folders of log file : {}", e);
		}
	}

	let logfile_path_for_dispatch = logfile_path.clone();

	charlie_buffalo::concurrent_logger_from(charlie_buffalo::Logger::new(
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
	))
}

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
