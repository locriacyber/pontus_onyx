#[actix_web::delete("/{requested_item:.*}")]
pub async fn delete_item(
	path: actix_web::web::Path<String>,
	request: actix_web::web::HttpRequest,
	database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<pontus_onyx::Database>>>,
) -> actix_web::web::HttpResponse {
	let if_match_result = if let Some(find_match) = request.headers().get("If-Match") {
		let find_match = find_match.to_str().unwrap().trim().replace('"', "");

		if let Ok(Some(pontus_onyx::Item::Document {
			etag: document_etag,
			content: _,
		})) = database.lock().unwrap().read(&path)
		{
			document_etag == find_match
		} else {
			true
		}
	} else {
		true
	};

	if if_match_result {
		match database.lock().unwrap().delete(&path) {
			Ok(etag) => {
				return actix_web::HttpResponse::Ok()
					.content_type("application/ld+json")
					.header("ETag", etag)
					.finish();
			}
			Err(pontus_onyx::DeleteError::WrongPath) => {
				return actix_web::HttpResponse::BadRequest()
					.content_type("application/ld+json")
					.body(r#"{"http_code":400,"http_description":"bad request"}"#);
			}
			Err(pontus_onyx::DeleteError::FolderDocumentConflict) => {
				return actix_web::HttpResponse::Conflict()
					.content_type("application/ld+json")
					.body(r#"{"http_code":409,"http_description":"conflict"}"#);
			}
			Err(pontus_onyx::DeleteError::NotFound) => {
				return actix_web::HttpResponse::NotFound()
					.content_type("application/ld+json")
					.body(r#"{"http_code":404,"http_description":"requested content not found"}"#);
			}
			Err(_todo) => {
				return actix_web::HttpResponse::InternalServerError()
					.content_type("application/ld+json")
					.body(r#"{"http_code":500,"http_description":"internal server error"}"#);
			}
		}
	} else {
		return actix_web::HttpResponse::PreconditionFailed().finish();
	}
}
