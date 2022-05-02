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
	request: actix_web::HttpRequest,
	path: actix_web::web::Path<String>,
	database: actix_web::web::Data<
		std::sync::Arc<std::sync::Mutex<pontus_onyx::database::Database>>,
	>,
) -> impl actix_web::Responder {
	let mut content = actix_web::web::BytesMut::new();
	while let Some(request_body) = futures::StreamExt::next(&mut request_payload).await {
		let request_body = request_body.unwrap();
		content.extend_from_slice(&request_body);
	}
	let content = content.freeze();

	let content_type = request.headers().get("content-type");

	// TODO : check security issue about this ?
	let all_origins = actix_web::http::header::HeaderValue::from_bytes(b"*").unwrap();
	let origin = request
		.headers()
		.get(actix_web::http::header::ORIGIN)
		.unwrap_or(&all_origins)
		.to_str()
		.unwrap();

	if content_type.is_none() {
		return pontus_onyx::database::build_http_json_response(
			origin,
			request.method(),
			actix_web::http::StatusCode::BAD_REQUEST,
			None,
			Some(String::from("missing content-type HTTP header")),
			true,
		);
	}

	match database.lock().unwrap().put(
		&pontus_onyx::item::ItemPath::from(path.into_inner().as_str()),
		pontus_onyx::item::Item::Document {
			etag: pontus_onyx::item::Etag::from(""),
			content: Some(content.to_vec()),
			content_type: pontus_onyx::item::ContentType::from(
				content_type.unwrap().to_str().unwrap(),
			),
			last_modified: chrono::Utc::now(),
		},
		super::convert_actix_if_match(&request)
			.first()
			.unwrap_or(&pontus_onyx::item::Etag::from("")),
		&super::convert_actix_if_none_match(&request)
			.iter()
			.collect::<Vec<&pontus_onyx::item::Etag>>(),
	) {
		pontus_onyx::database::PutResult::Created(new_etag) => {
			return pontus_onyx::database::build_http_json_response(
				origin,
				request.method(),
				actix_web::http::StatusCode::CREATED,
				Some(new_etag),
				None,
				true,
			);
		}
		pontus_onyx::database::PutResult::Updated(new_etag) => {
			return pontus_onyx::database::build_http_json_response(
				origin,
				request.method(),
				actix_web::http::StatusCode::OK,
				Some(new_etag),
				None,
				true,
			);
		}
		pontus_onyx::database::PutResult::Err(e) => {
			if e.is::<pontus_onyx::database::sources::memory::PutError>() {
				pontus_onyx::database::Error::to_response(
					&*e.downcast::<pontus_onyx::database::sources::memory::PutError>()
						.unwrap(),
					origin,
					true,
				)
			} else if e.is::<pontus_onyx::database::sources::folder::PutError>() {
				pontus_onyx::database::Error::to_response(
					&*e.downcast::<pontus_onyx::database::sources::folder::PutError>()
						.unwrap(),
					origin,
					true,
				)
			} else {
				pontus_onyx::database::build_http_json_response(
					origin,
					request.method(),
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
					None,
					true,
				)
			}
		}
	}
}

#[cfg(test)]
mod tests;
