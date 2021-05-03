use std::convert::TryFrom;

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
	request: actix_web::web::HttpRequest,
	form: actix_web::web::Form<OauthPostQuery>,
	form_tokens: actix_web::web::Data<
		std::sync::Arc<std::sync::Mutex<Vec<crate::http_server::OauthFormToken>>>,
	>,
	access_tokens: actix_web::web::Data<
		std::sync::Arc<std::sync::Mutex<Vec<crate::http_server::AccessBearer>>>,
	>,
) -> actix_web::Result<actix_web::web::HttpResponse> {
	let _host = request.headers().get("host");
	let origin = request.headers().get("origin");
	let _referer = request.headers().get("referer");

	match origin {
		Some(path) => {
			if path != "http://localhost:7541" {
				println!("security issue : wrong origin : {:?}", path);

				return Ok(actix_web::HttpResponse::Found()
					.header(
						actix_web::http::header::LOCATION,
						format!(
							"/oauth/{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
							form.username,
							percent_encoding::percent_encode(
								percent_encoding::percent_decode(
									form.redirect_uri.as_bytes()
								)
									.decode_utf8()
									.unwrap()
									.as_bytes(),
								percent_encoding::NON_ALPHANUMERIC
							),
							form.scope,
							form.client_id,
							form.response_type,
							"security_issue"
						),
					)
					.finish());
			}
		}
		None => {
			println!("security issue : no origin");

			return Ok(actix_web::HttpResponse::Found()
				.header(
					actix_web::http::header::LOCATION,
					format!(
						"/oauth/{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
						form.username,
						percent_encoding::percent_encode(
							percent_encoding::percent_decode(
								form.redirect_uri.as_bytes()
							)
								.decode_utf8()
								.unwrap()
								.as_bytes(),
							percent_encoding::NON_ALPHANUMERIC
						),
						form.scope,
						form.client_id,
						form.response_type,
						"security_issue"
					),
				)
				.finish());
		}
	}

	let token = percent_encoding::percent_decode(form.token.as_bytes())
		.decode_utf8()
		.unwrap();

	let form_tokens = form_tokens.lock().unwrap();
	let token_search = form_tokens.iter().find(|e| e.value() == token);
	match token_search {
		Some(token_found) => {
			if token_found.has_expirated() {
				println!("security issue : expirated token : {:?}", token_found);

				return Ok(actix_web::HttpResponse::Found()
					.header(
						actix_web::http::header::LOCATION,
						format!(
							"/oauth/{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
							form.username,
							percent_encoding::percent_encode(
								percent_encoding::percent_decode(
									form.redirect_uri.as_bytes()
								)
									.decode_utf8()
									.unwrap()
									.as_bytes(),
								percent_encoding::NON_ALPHANUMERIC
							),
							form.scope,
							form.client_id,
							form.response_type,
							"security_issue"
						),
					)
					.finish());
			}
		}
		None => {
			println!("security issue : token not found");

			return Ok(actix_web::HttpResponse::Found()
				.header(
					actix_web::http::header::LOCATION,
					format!(
						"/oauth/{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
						form.username,
						percent_encoding::percent_encode(
							percent_encoding::percent_decode(
								form.redirect_uri.as_bytes()
							)
								.decode_utf8()
								.unwrap()
								.as_bytes(),
							percent_encoding::NON_ALPHANUMERIC
						),
						form.scope,
						form.client_id,
						form.response_type,
						"security_issue"
					),
				)
				.finish());
		}
	}

	if form.allow == "Allow" {
		std::thread::sleep(std::time::Duration::from_secs(0)); // TODO : anti brute-force
		if form.username == "todo" && form.password == "todo" {
			// TODO : what if form.redirect_uri already contains fragment `#something` ?

			let scopes = percent_encoding::percent_decode(form.scope.as_bytes())
				.decode_utf8()
				.unwrap()
				.split(',')
				.map(|e| crate::http_server::Scope::try_from(e.trim()).unwrap())
				.collect();

			let new_token = crate::http_server::AccessBearer::new(
				scopes,
				&percent_encoding::percent_decode(form.client_id.as_bytes())
					.decode_utf8()
					.unwrap(),
				&percent_encoding::percent_decode(form.username.as_bytes())
					.decode_utf8()
					.unwrap(),
			);
			access_tokens.lock().unwrap().push(new_token.clone());

			let redirect = format!(
				"{}#access_token={}&token_type={}",
				percent_encoding::percent_decode(form.redirect_uri.as_bytes())
					.decode_utf8()
					.unwrap(),
				percent_encoding::percent_encode(
					new_token.name().as_bytes(),
					percent_encoding::NON_ALPHANUMERIC
				),
				"bearer"
			);

			Ok(actix_web::HttpResponse::Found()
				.header(actix_web::http::header::LOCATION, redirect)
				.finish()) // todo : some text for users ?
		} else {
			println!("security issue : wrong credentials");

			Ok(actix_web::HttpResponse::Found()
				.header(
					actix_web::http::header::LOCATION,
					format!(
						"/oauth/{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
						form.username,
						percent_encoding::percent_encode(
							percent_encoding::percent_decode(
								form.redirect_uri.as_bytes()
							)
								.decode_utf8()
								.unwrap()
								.as_bytes(),
							percent_encoding::NON_ALPHANUMERIC
						),
						form.scope,
						form.client_id,
						form.response_type,
						"wrong_credentials"
					),
				)
				.finish()) // todo : some text for users ?
		}
	} else {
		println!("security issue : not allowed");

		Ok(actix_web::HttpResponse::Found()
			.header(
				actix_web::http::header::LOCATION,
				format!(
					"/oauth/{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
					form.username,
					percent_encoding::percent_encode(
						percent_encoding::percent_decode(
							form.redirect_uri.as_bytes()
						)
							.decode_utf8()
							.unwrap()
							.as_bytes(),
						percent_encoding::NON_ALPHANUMERIC
					),
					form.scope,
					form.client_id,
					form.response_type,
					"security_issue"
				),
			)
			.finish()) // todo : some text for users ?
	}
}
