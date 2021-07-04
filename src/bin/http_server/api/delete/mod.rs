#[actix_web::delete("/storage/{requested_item:.*}")]
pub async fn delete_item(
	path: actix_web::web::Path<String>,
	request: actix_web::web::HttpRequest,
	database: actix_web::web::Data<
		std::sync::Arc<std::sync::Mutex<pontus_onyx::database::Database>>,
	>,
) -> impl actix_web::Responder {
	// TODO : check security issue about this ?
	let all_origins = actix_web::http::HeaderValue::from_bytes(b"*").unwrap();
	let origin = request
		.headers()
		.get(actix_web::http::header::ORIGIN)
		.unwrap_or(&all_origins)
		.to_str()
		.unwrap();

	match database.lock().unwrap().delete(
		&std::path::PathBuf::from(path.to_string()),
		super::convert_actix_if_match(&request)
			.first()
			.unwrap_or(&&pontus_onyx::Etag::from("")),
	) {
		Ok(etag) => {
			return pontus_onyx::database::build_http_json_response(
				origin,
				request.method(),
				actix_web::http::StatusCode::OK,
				Some(etag),
				None,
				true,
			);
		}
		Err(e) => {
			if e.is::<pontus_onyx::database::memory::DeleteError>() {
				pontus_onyx::database::Error::to_response(
					&*e.downcast::<pontus_onyx::database::memory::DeleteError>()
						.unwrap(),
					origin,
					true,
				)
			} else if e.is::<pontus_onyx::database::folder::DeleteError>() {
				pontus_onyx::database::Error::to_response(
					&*e.downcast::<pontus_onyx::database::folder::DeleteError>()
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
