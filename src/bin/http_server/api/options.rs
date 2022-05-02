/*
TODO :
	A successful OPTIONS request SHOULD be responded to as described in
	the CORS section below.
*/
/*
TODO :
	The server MUST also
	reply to preflight OPTIONS requests as per CORS.
*/
#[actix_web::options("/storage/{requested_item:.*}")]
pub async fn options_item(
	_path: actix_web::web::Path<String>,
	request: actix_web::HttpRequest,
) -> impl actix_web::Responder {
	// TODO : check security issue about this ?
	let all_origins = actix_web::http::header::HeaderValue::from_bytes(b"*").unwrap();
	let origin = request
		.headers()
		.get(actix_web::http::header::ORIGIN)
		.unwrap_or(&all_origins)
		.to_str()
		.unwrap();

	// TODO ; build at the end of the implementation.
	let mut response = actix_web::HttpResponse::Ok();
	response.insert_header((actix_web::http::header::CACHE_CONTROL, "no-cache"));
	response.insert_header((actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin));

	if origin != "*" {
		response.insert_header((actix_web::http::header::VARY, "Origin"));
	}

	response.insert_header((
		actix_web::http::header::ACCESS_CONTROL_ALLOW_METHODS,
		"OPTIONS, GET, HEAD, PUT, DELETE",
	));
	response.insert_header((
		actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
		"Content-Length, Content-Type, Etag",
	));
	response.insert_header((
		actix_web::http::header::ACCESS_CONTROL_ALLOW_HEADERS,
		"Authorization, Content-Length, Content-Type, Origin, If-Match, If-None-Match",
	));

	return response.finish();
}
