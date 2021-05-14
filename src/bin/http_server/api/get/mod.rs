#[actix_web::get("/storage/{requested_item:.*}")]
pub async fn get_item(
	path: actix_web::web::Path<String>,
	request: actix_web::web::HttpRequest,
	database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<pontus_onyx::Database>>>,
) -> actix_web::web::HttpResponse {
	/* TODO :
	let if_match: Result<actix_web::http::header::IfMatch, actix_web::error::ParseError> = actix_web::http::header::Header::parse(&request);
	let if_none_match: Result<actix_web::http::header::IfNoneMatch, actix_web::error::ParseError> = actix_web::http::header::IfNoneMatch::parse(&request);
	*/

	let if_none_match = request
		.headers()
		.get("If-None-Match")
		.map(|e| (e.to_str().unwrap()).split(',').collect::<Vec<&str>>());

	match database.lock().unwrap().get(
		&path,
		request
			.headers()
			.get("If-Match")
			.map(|e| e.to_str().unwrap()),
		if_none_match,
	) {
		Ok(pontus_onyx::Item::Document {
			etag,
			content,
			content_type,
			..
		}) => {
			return actix_web::HttpResponse::Ok()
				.header("ETag", etag.clone())
				.header("Cache-Control", "no-cache")
				.header("Access-Control-Allow-Origin", "*")
				.content_type(content_type)
				.body(content.clone());
		}
		Ok(pontus_onyx::Item::Folder {
			etag: folder_etag,
			content,
		}) => {
			let mut items_result = serde_json::json!({});
			for (child_name, child) in content.iter().filter(|(_, e)| match &***e {
				pontus_onyx::Item::Document { .. } => true,
				pontus_onyx::Item::Folder { .. } => !e.is_empty(),
			}) {
				match &**child {
					pontus_onyx::Item::Folder { etag, content: _ } => {
						items_result[format!("{}/", child_name)] = serde_json::json!({
							"ETag": etag,
						});
					}
					pontus_onyx::Item::Document {
						etag,
						content: document_content,
						content_type,
						last_modified,
					} => {
						items_result[child_name] = serde_json::json!({
							"ETag": etag,
							"Content-Type": content_type,
							"Content-Length": document_content.len(),
							"Last-Modified": last_modified.format(crate::http_server::RFC5322).to_string(),
						});
					}
				}
			}

			return actix_web::HttpResponse::Ok()
				.content_type("application/ld+json")
				.header("ETag", folder_etag.clone())
				.header("Cache-Control", "no-cache")
				.header("Access-Control-Allow-Origin", "*")
				.body(
					serde_json::json!({
						"@context": "http://remotestorage.io/spec/folder-description",
						"items": items_result,
					})
					.to_string(),
				);
		}
		Err(e) => actix_web::HttpResponse::from(e),
	}
}

#[cfg(test)]
mod tests;
