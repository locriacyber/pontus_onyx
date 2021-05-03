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
pub async fn options_item(_path: actix_web::web::Path<String>) -> actix_web::web::HttpResponse {
	// TODO ; build at the end of the implementation.
	return actix_web::HttpResponse::Ok()
		.header("Cache-Control", "no-cache")
		.header("Access-Control-Allow-Origin", "*")
		.header("Access-Control-Allow-Methods", "GET, PUT, DELETE")
		.header(
			"Access-Control-Allow-Headers",
			"Authorization, Content-Length, Content-Type, Origin, If-Match, If-None-Match",
		)
		.finish();
}
