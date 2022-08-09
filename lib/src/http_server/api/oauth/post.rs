use std::convert::TryFrom;
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize)]
pub struct OauthPostQuery {
	redirect_uri: String,
	scope: String,
	client_id: String,
	response_type: String,
	username: String,
	password: String,
	allow: String,
	token: String,
}

#[actix_web::post("/oauth")]
pub async fn post_oauth(
	request: actix_web::HttpRequest,
	form: actix_web::web::Form<OauthPostQuery>,
	form_tokens: actix_web::web::Data<
		Arc<Mutex<Vec<crate::http_server::middlewares::OauthFormToken>>>,
	>,
	access_tokens: actix_web::web::Data<Arc<Mutex<Vec<crate::http_server::AccessBearer>>>>,
	users: actix_web::web::Data<Arc<Mutex<crate::http_server::Users>>>,
	settings: actix_web::web::Data<Arc<Mutex<crate::http_server::Settings>>>,
	program_state: actix_web::web::Data<Arc<Mutex<crate::http_server::ProgramState>>>,
	logger: actix_web::web::Data<Arc<Mutex<charlie_buffalo::Logger>>>,
) -> actix_web::Result<actix_web::HttpResponse> {
	let _host = request.headers().get("host");
	let origin = request.headers().get("origin");
	let _referer = request.headers().get("referer");

	match origin {
		Some(path) => {
			let settings = settings.lock().unwrap().clone();

			let mut allowed_domains = vec![];
			// TODO : probably a security issue :
			allowed_domains.push(format!("http://localhost:{}", settings.port));
			// TODO : probably a security issue :
			if settings.port == 80 {
				allowed_domains.push(String::from("http://localhost"));
			}
			if program_state.lock().unwrap().https_mode {
				if let Some(https) = settings.https.clone() {
					allowed_domains.push(format!("https://localhost:{}", https.port));
					if https.port != 443 {
						allowed_domains.push(String::from("https://localhost"));
					}
				}
			}
			if let Some(domain) = settings.domain {
				// TODO : probably a security issue :
				allowed_domains.push(format!("http://{}:{}", domain, settings.port));
				// TODO : probably a security issue :
				if settings.port == 80 {
					allowed_domains.push(format!("http://{}", domain));
				}
				if program_state.lock().unwrap().https_mode {
					if let Some(https) = settings.https.clone() {
						allowed_domains.push(format!("https://{}:{}", domain, https.port));
						if https.port != 443 {
							allowed_domains.push(format!("https://{}", domain));
						}
					}
				}
			}

			if !allowed_domains.contains(&String::from(path.to_str().unwrap_or_default())) {
				logger.lock().unwrap().push(
					vec![
						(String::from("event"), String::from("oauth_submit")),
						(String::from("level"), String::from("ERROR")),
					],
					Some(&format!("wrong origin : {:?}", path)),
				);

				return Ok(actix_web::HttpResponse::Found()
					.insert_header((
						actix_web::http::header::LOCATION,
						format!(
							"/oauth/{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
							form.username,
							pct_str::PctString::encode(
								pct_str::PctString::new(
									&form.redirect_uri
								)
									.unwrap()
									.decode()
									.chars(),
								pct_str::URIReserved
							),
							form.scope,
							form.client_id,
							form.response_type,
							"security_issue"
						),
					))
					.finish());
			}
		}
		None => {
			logger.lock().unwrap().push(
				vec![
					(String::from("event"), String::from("oauth_submit")),
					(String::from("level"), String::from("ERROR")),
				],
				Some("no origin"),
			);

			return Ok(actix_web::HttpResponse::Found()
				.insert_header((
					actix_web::http::header::LOCATION,
					format!(
						"/oauth/{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
						form.username,
						pct_str::PctString::encode(
							pct_str::PctString::new(
								&form.redirect_uri
							)
								.unwrap()
								.decode()
								.chars(),
							pct_str::URIReserved
						),
						form.scope,
						form.client_id,
						form.response_type,
						"security_issue"
					),
				))
				.finish());
		}
	}

	let token = pct_str::PctString::new(&form.token).unwrap().decode();

	let form_tokens = form_tokens.lock().unwrap();
	let token_search = form_tokens.iter().find(|e| e.get_value() == token);
	match token_search {
		Some(token_found) => {
			if token_found.has_expirated() {
				logger.lock().unwrap().push(
					vec![
						(String::from("event"), String::from("oauth_submit")),
						(String::from("level"), String::from("ERROR")),
					],
					Some(&format!("expirated form token : {:?}", token_found)),
				);

				return Ok(actix_web::HttpResponse::Found()
					.insert_header((
						actix_web::http::header::LOCATION,
						format!(
							"/oauth/{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
							form.username,
							pct_str::PctString::encode(
								pct_str::PctString::new(
									&form.redirect_uri
								)
									.unwrap()
									.decode()
									.chars(),
								pct_str::URIReserved
							),
							form.scope,
							form.client_id,
							form.response_type,
							"security_issue"
						),
					))
					.finish());
			}
		}
		None => {
			logger.lock().unwrap().push(
				vec![
					(String::from("event"), String::from("oauth_submit")),
					(String::from("level"), String::from("ERROR")),
				],
				Some("token not found"),
			);

			return Ok(actix_web::HttpResponse::Found()
				.insert_header((
					actix_web::http::header::LOCATION,
					format!(
						"/oauth/{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
						form.username,
						pct_str::PctString::encode(
							pct_str::PctString::new(
								&form.redirect_uri
							)
								.unwrap()
								.decode()
								.chars(),
							pct_str::URIReserved
						),
						form.scope,
						form.client_id,
						form.response_type,
						"security_issue"
					),
				))
				.finish());
		}
	}

	if form.allow == "Allow" {
		std::thread::sleep(std::time::Duration::from_secs(
			settings
				.lock()
				.unwrap()
				.oauth_wait_seconds
				.unwrap_or_else(|| {
					crate::http_server::Settings::new(std::path::PathBuf::from("."))
						.oauth_wait_seconds
						.unwrap()
				}),
		));
		if users
			.lock()
			.unwrap()
			.check(&form.username, &mut String::from(&form.password))
		{
			// TODO : what if form.redirect_uri already contains fragment `#something` ?

			let scopes = pct_str::PctString::new(&form.scope)
				.unwrap()
				.decode()
				.split(' ')
				.map(|e| crate::scope::Scope::try_from(e.trim()).unwrap())
				.collect();

			let new_token = crate::http_server::AccessBearer::new(
				scopes,
				&pct_str::PctString::new(&form.client_id).unwrap().decode(),
				&pct_str::PctString::new(&form.username).unwrap().decode(),
			);
			access_tokens.lock().unwrap().push(new_token.clone());

			let redirect = format!(
				"{}#access_token={}&token_type={}",
				pct_str::PctString::new(&form.redirect_uri)
					.unwrap()
					.decode(),
				pct_str::PctString::encode(new_token.get_name().chars(), pct_str::URIReserved),
				"bearer"
			);

			Ok(actix_web::HttpResponse::Found()
				.insert_header((actix_web::http::header::LOCATION, redirect))
				.finish()) // todo : some text for users ?
		} else {
			logger.lock().unwrap().push(
				vec![
					(String::from("event"), String::from("oauth_submit")),
					(String::from("level"), String::from("ERROR")),
				],
				Some("wrong credentials"),
			);

			Ok(actix_web::HttpResponse::Found()
				.insert_header((
					actix_web::http::header::LOCATION,
					format!(
						"/oauth/{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
						form.username,
						pct_str::PctString::encode(
							pct_str::PctString::new(
								&form.redirect_uri
							)
								.unwrap()
								.decode()
								.chars(),
							pct_str::URIReserved
						),
						form.scope,
						form.client_id,
						form.response_type,
						"wrong_credentials"
					),
				))
				.finish()) // todo : some text for users ?
		}
	} else {
		logger.lock().unwrap().push(
			vec![
				(String::from("event"), String::from("oauth_submit")),
				(String::from("level"), String::from("ERROR")),
			],
			Some("not allowed"),
		);

		Ok(actix_web::HttpResponse::Found()
			.insert_header((
				actix_web::http::header::LOCATION,
				format!(
					"/oauth/{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
					form.username,
					pct_str::PctString::encode(
						pct_str::PctString::new(
							&form.redirect_uri
						)
							.unwrap()
							.decode()
							.chars(),
						pct_str::URIReserved
					),
					form.scope,
					form.client_id,
					form.response_type,
					"security_issue"
				),
			))
			.finish()) // todo : some text for users ?
	}
}
