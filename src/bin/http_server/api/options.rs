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
#[actix_web::options("/{requested_item:.*}")]
pub async fn options_item(_path: actix_web::web::Path<String>) -> actix_web::web::HttpResponse {
	// TODO ; build at the end of the implementation.
	todo!()
}
