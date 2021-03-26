#[actix_web::get("/.well-known/webfinger")]
pub async fn webfinger_handle(
	_query: actix_web::web::Query<WebfingerQuery>,
) -> actix_web::web::HttpResponse {
	actix_web::HttpResponse::Ok()
		.content_type("application/ld+json")
		.body(
			format!(r#"{{"href":"/","rel":"http://tools.ietf.org/id/draft-dejong-remotestorage","properties":{{"http://remotestorage.io/spec/version":"{}","http://tools.ietf.org/html/rfc6749#section-4.2":{}}}}}"#,
				"draft-dejong-remotestorage-16",
				"TODO"
			)
		)
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
pub struct WebfingerQuery {}
