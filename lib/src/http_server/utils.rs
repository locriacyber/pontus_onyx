pub fn build_server_address(
	settings: &crate::http_server::Settings,
	program_state: &super::ProgramState,
) -> String {
	let localhost = String::from("localhost");

	let mut protocol = String::from("http");
	if let Some(force_https) = settings.force_https {
		if force_https {
			protocol += "s";
		}
	} else if program_state.https_mode {
		protocol += "s";
	}

	let mut domain = settings
		.domain
		.as_ref()
		.unwrap_or_else(|| &localhost)
		.clone();
	if let Some(force_domain) = &settings.domain {
		if !force_domain.trim().is_empty() {
			domain = force_domain.clone();
		}
	}

	let port = if let Some(force_https) = &settings.force_https {
		if *force_https {
			if let Some(https) = &settings.https {
				if https.port != 443 {
					format!(":{}", https.port)
				} else {
					String::new()
				}
			} else if settings.port != 80 {
				format!(":{}", settings.port)
			} else {
				String::new()
			}
		} else if program_state.https_mode {
			let https = settings.https.clone().unwrap();
			if https.port != 443 {
				format!(":{}", https.port)
			} else {
				String::new()
			}
		} else if settings.port != 80 {
			format!(":{}", settings.port)
		} else {
			String::new()
		}
	} else if program_state.https_mode {
		let https = settings.https.clone().unwrap();
		if https.port != 443 {
			format!(":{}", https.port)
		} else {
			String::new()
		}
	} else if settings.port != 80 {
		format!(":{}", settings.port)
	} else {
		String::new()
	};

	let mut domain_suffix = String::new();
	if let Some(suffix) = &settings.domain_suffix {
		if !suffix.trim().is_empty() && !suffix.trim().ends_with('/') {
			domain_suffix = format!("{}/", suffix.trim())
		} else {
			domain_suffix = String::from(suffix.trim())
		}
	}

	format!("{}://{}{}/{}", protocol, domain, port, domain_suffix)
}

#[test]
fn pbw1cgzctiqe163() {
	let settings = super::Settings::new(tempfile::tempdir().unwrap().into_path());
	let state = super::ProgramState { https_mode: false };

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}:{}/{}",
			"http",
			settings.domain.unwrap_or_else(|| String::from("localhost")),
			settings.port,
			""
		)
	);
}

#[test]
fn ykf0gcnr7z2ko4wtx8uub() {
	let mut settings = super::Settings::new(tempfile::tempdir().unwrap().into_path());
	settings.domain_suffix = Some(String::from("test"));
	let state = super::ProgramState { https_mode: false };

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}:{}/{}",
			"http",
			settings.domain.unwrap_or_else(|| String::from("localhost")),
			settings.port,
			"test/"
		)
	);
}

#[test]
fn wxpy6tncuwbbavvxi() {
	let mut settings = super::Settings::new(tempfile::tempdir().unwrap().into_path());
	settings.domain_suffix = Some(String::from("test/"));
	let state = super::ProgramState { https_mode: false };

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}:{}/{}",
			"http",
			settings.domain.unwrap_or_else(|| String::from("localhost")),
			settings.port,
			"test/"
		)
	);
}

#[test]
fn fpfxwrixa1jz7t() {
	let settings = super::Settings::new(tempfile::tempdir().unwrap().into_path());

	let state = super::ProgramState { https_mode: true };

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}:{}/{}",
			"https",
			settings.domain.unwrap_or_else(|| String::from("localhost")),
			settings.https.unwrap().port,
			""
		)
	);
}

#[test]
fn xtgfpc3x1zcmb() {
	let domain = String::from("example.com");
	let mut settings = super::Settings::new(tempfile::tempdir().unwrap().into_path());
	settings.domain = Some(domain.clone());
	let state = super::ProgramState { https_mode: false };

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}/{}",
			"http",
			format!("{}:{}", domain, settings.port),
			""
		)
	);
}

#[test]
fn ekkvpuijzifxc() {
	let domain = String::from("example.com");
	let mut settings = super::Settings::new(tempfile::tempdir().unwrap().into_path());
	settings.domain = Some(domain.clone());
	let state = super::ProgramState { https_mode: true };

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}/{}",
			"https",
			format!("{}:{}", domain, settings.https.unwrap().port),
			""
		)
	);
}

#[test]
fn bj8n5zhu2oaaed55561ygk() {
	let domain = String::from("example.com");
	let mut settings = super::Settings::new(tempfile::tempdir().unwrap().into_path());
	settings.domain = Some(domain.clone());
	settings.port = 80;
	if let Some(https) = &mut settings.https {
		https.port = 443;
	}
	let state = super::ProgramState { https_mode: false };

	assert_eq!(
		build_server_address(&settings, &state),
		format!("{}://{}/{}", "http", domain, "")
	);
}

#[test]
fn d434yaaxfqcnd4j() {
	let domain = String::from("example.com");
	let mut settings = super::Settings::new(tempfile::tempdir().unwrap().into_path());
	settings.domain = Some(domain.clone());
	settings.port = 80;
	if let Some(https) = &mut settings.https {
		https.port = 443;
	}
	let state = super::ProgramState { https_mode: true };

	assert_eq!(
		build_server_address(&settings, &state),
		format!("{}://{}/{}", "https", domain, "")
	);
}
