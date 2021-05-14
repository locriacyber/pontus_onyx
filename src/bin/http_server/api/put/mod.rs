/*
TODO :
	Unless [KERBEROS] is used (see section 10 below), all other
	requests SHOULD present a bearer token with sufficient access scope,
	using a header of the following form (no double quotes here):
		Authorization: Bearer <access_token>
*/
#[actix_web::put("/storage/{requested_item:.*}")]
pub async fn put_item(
	mut request_payload: actix_web::web::Payload,
	request: actix_web::web::HttpRequest,
	path: actix_web::web::Path<String>,
	database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<pontus_onyx::Database>>>,
) -> actix_web::web::HttpResponse {
	let mut content = actix_web::web::BytesMut::new();
	while let Some(request_body) = futures::StreamExt::next(&mut request_payload).await {
		let request_body = request_body.unwrap();
		content.extend_from_slice(&request_body);
	}
	let content = content.freeze();

	let content_type = request.headers().get("content-type");

	if content_type.is_none() {
		return pontus_onyx::build_http_json_response(
			request.method(),
			actix_web::http::StatusCode::BAD_REQUEST,
			None,
			Some(String::from("missing content-type HTTP header")),
			true,
		);
	}

	let if_none_match = request
		.headers()
		.get("If-None-Match")
		.map(|e| (e.to_str().unwrap()).split(',').collect::<Vec<&str>>());

	match database.lock().unwrap().put(
		&path,
		pontus_onyx::Item::Document {
			etag: String::new(),
			content: content.to_vec(),
			content_type: String::from(content_type.unwrap().to_str().unwrap()),
			last_modified: chrono::Utc::now(),
		},
		request
			.headers()
			.get("If-Match")
			.map(|e| e.to_str().unwrap()),
		if_none_match,
	) {
		pontus_onyx::database::ResultPut::Created(new_etag) => {
			return pontus_onyx::build_http_json_response(
				request.method(),
				actix_web::http::StatusCode::CREATED,
				Some(new_etag),
				None,
				true,
			);
		}
		pontus_onyx::database::ResultPut::Updated(new_etag) => {
			return pontus_onyx::build_http_json_response(
				request.method(),
				actix_web::http::StatusCode::OK,
				Some(new_etag),
				None,
				true,
			);
		}
		pontus_onyx::database::ResultPut::Err(e) => actix_web::HttpResponse::from(e),
	}
}

#[cfg(test)]
mod tests;
