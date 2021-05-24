#[actix_web::get("/storage/{requested_item:.*}")]
pub async fn get_item(
	path: actix_web::web::Path<String>,
	request: actix_web::web::HttpRequest,
	database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<pontus_onyx::Database>>>,
) -> actix_web::web::HttpResponse {
	match database.lock().unwrap().get(
		&path,
		super::convert_actix_if_match(&request)
			.first()
			.unwrap_or(&String::new()),
		super::convert_actix_if_none_match(&request),
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
					pontus_onyx::Item::Folder { etag, .. } => {
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
