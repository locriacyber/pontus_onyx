#[actix_web::get("/storage/{requested_item:.*}")]
pub async fn get_item(
	path: actix_web::web::Path<String>,
	request: actix_web::web::HttpRequest,
	database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<pontus_onyx::Database>>>,
) -> actix_web::web::HttpResponse {
	// TODO : check security issue about this ?
	let all_origins = actix_web::http::HeaderValue::from_bytes(b"*").unwrap();
	let origin = request
		.headers()
		.get(actix_web::http::header::ORIGIN)
		.unwrap_or(&all_origins)
		.to_str()
		.unwrap();

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
			let mut response = actix_web::HttpResponse::Ok();
			response.header(actix_web::http::header::ETAG, etag.clone());
			response.header(actix_web::http::header::CACHE_CONTROL, "no-cache");
			response.header(actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);

			if origin != "*" {
				response.header(actix_web::http::header::VARY, "Origin");
			}

			response.header(
				actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
				"Content-Length, Content-Type, Etag",
			);
			response.content_type(content_type);

			return response.body(content.clone());
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

			let mut response = actix_web::HttpResponse::Ok();
			response.content_type("application/ld+json");
			response.header(actix_web::http::header::ETAG, folder_etag.clone());
			response.header(actix_web::http::header::CACHE_CONTROL, "no-cache");
			response.header(actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);

			if origin != "*" {
				response.header(actix_web::http::header::VARY, "Origin");
			}

			response.header(
				actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
				"Content-Length, Content-Type, Etag",
			);

			return response.body(
				serde_json::json!({
					"@context": "http://remotestorage.io/spec/folder-description",
					"items": items_result,
				})
				.to_string(),
			);
		}
		Err(e) => e.to_response(origin, true),
	}
}

#[cfg(test)]
mod tests;
