#[actix_web::get("/.well-known/webfinger")]
pub async fn webfinger_handle(
	query: actix_web::web::Query<WebfingerQuery>,
	settings: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<super::Settings>>>,
	program_state: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<crate::ProgramState>>>,
) -> actix_web::web::HttpResponse {
	let default_body = format!(
		r#"{{"href":"/","rel":"http://tools.ietf.org/id/draft-dejong-remotestorage","properties":{{"http://remotestorage.io/spec/version":"{}","http://tools.ietf.org/html/rfc6749#section-4.2":"{}"}}}}"#,
		"draft-dejong-remotestorage-16", "TODO"
	);

	match &query.resource {
		Some(resource) if resource.starts_with("acct:") => {
			let resource = resource.strip_prefix("acct:").unwrap();
			let items = resource.split('@').collect::<Vec<&str>>();
			if items.len() == 2 {
				let user = items.get(0).unwrap();
				let _domain = items.get(1).unwrap();
				// todo : check if user exists ?
				// todo : check domain & host header ?

				let server_addr = if program_state.lock().unwrap().https_mode {
					format!("https://localhost:{}/", settings.lock().unwrap().https.port)
				} else {
					format!("http://localhost:{}/", settings.lock().unwrap().port)
				};

				actix_web::HttpResponse::Ok()
					.header("Access-Control-Allow-Origin", "*")
					.content_type("application/ld+json")
					.body(format!(
						r#"{{"links":[{{"href":"{}","rel":"{}","properties":{{"{}":"{}","{}":"{}","{}":{},"{}":{},"{}":{}}}}}]}}"#,
						format!("{}storage/{}", server_addr, user),
						"http://tools.ietf.org/id/draft-dejong-remotestorage",
						"http://remotestorage.io/spec/version",
						"draft-dejong-remotestorage-16",
						"http://tools.ietf.org/html/rfc6749#section-4.2",
						format!("{}oauth/{}", server_addr, user),
						"http://tools.ietf.org/html/rfc6750#section-2.3", "null",
						"http://tools.ietf.org/html/rfc7233", "null",
						"http://remotestorage.io/spec/web-authoring", "null"
					))
			} else {
				actix_web::HttpResponse::Ok()
					.content_type("application/ld+json")
					.body(default_body)
			}
		}
		Some(_) => actix_web::HttpResponse::Ok()
			.content_type("application/ld+json")
			.body(default_body),
		None => actix_web::HttpResponse::Ok()
			.content_type("application/ld+json")
			.body(default_body),
	}

	/*
	TODO :
		If <auth-dialog> is a URL, the user can supply their credentials
		for accessing the account (how, is out of scope), and allow or
		reject a request by the connecting application to obtain a bearer
		token for a certain list of access scopes.
	*/
	/*
	TODO :
		Non-breaking examples that have been proposed so far, include a
		"http://tools.ietf.org/html/rfc6750#section-2.3" property, set to
		the string value "true" if the server supports passing the bearer
		token in the URI query parameter as per section 2.3 of [BEARER],
		instead of in the request header.
	*/
}

#[derive(serde::Deserialize)]
pub struct WebfingerQuery {
	resource: Option<String>,
}
