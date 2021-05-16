#[actix_web::delete("/storage/{requested_item:.*}")]
pub async fn delete_item(
	path: actix_web::web::Path<String>,
	request: actix_web::web::HttpRequest,
	database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<pontus_onyx::Database>>>,
) -> actix_web::web::HttpResponse {
	match database.lock().unwrap().delete(
		&path,
		super::convert_actix_if_match(&request)
			.first()
			.unwrap_or(&String::new()),
	) {
		Ok(etag) => {
			return pontus_onyx::database::build_http_json_response(
				request.method(),
				actix_web::http::StatusCode::OK,
				Some(etag),
				None,
				true,
			);
		}
		Err(e) => actix_web::HttpResponse::from(e),
	}
}

#[cfg(test)]
mod tests;
