#[actix_web::head("/storage/{requested_item:.*}")]
pub async fn head_item(
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

	match database.lock().unwrap().get(
		std::path::Path::new(&path.to_string()),
		super::convert_actix_if_match(&request)
			.first()
			.unwrap_or(&pontus_onyx::Etag::from("")),
		&super::convert_actix_if_none_match(&request)
			.iter()
			.collect::<Vec<&pontus_onyx::Etag>>(),
	) {
		Ok(pontus_onyx::Item::Document {
			etag, content_type, ..
		}) => {
			let etag: String = etag.into();
			let mut response = actix_web::HttpResponse::Ok();
			response.header(actix_web::http::header::ETAG, etag);
			response.header(actix_web::http::header::CACHE_CONTROL, "no-cache");
			response.header(actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);

			if origin != "*" {
				response.header(actix_web::http::header::VARY, "Origin");
			}

			let content_type: String = content_type.into();
			response.content_type(content_type);

			return response.finish();
		}
		Ok(pontus_onyx::Item::Folder {
			etag: folder_etag,
			content: Some(content),
		}) => {
			let mut items_result = serde_json::json!({});
			for (child_name, child) in content.iter().filter(|(_, e)| match &***e {
				pontus_onyx::Item::Document { .. } => true,
				pontus_onyx::Item::Folder {
					content: Some(content),
					..
				} => !content.is_empty(),
				pontus_onyx::Item::Folder { content: None, .. } => todo!(),
			}) {
				match &**child {
					pontus_onyx::Item::Folder { etag, .. } => {
						items_result[format!("{}/", child_name)] = serde_json::json!({
							"ETag": etag,
						});
					}
					pontus_onyx::Item::Document {
						etag,
						content: Some(document_content),
						content_type,
						last_modified,
					} => {
						let child_name: String = child_name.clone();
						items_result[child_name] = serde_json::json!({
							"ETag": etag,
							"Content-Type": content_type,
							"Content-Length": document_content.len(),
							"Last-Modified": last_modified.format(crate::http_server::RFC5322).to_string(),
						});
					}
					pontus_onyx::Item::Document { content: None, .. } => {
						return pontus_onyx::database::build_http_json_response(
							origin,
							request.method(),
							actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
							None,
							None,
							false,
						);
					}
				}
			}

			let folder_etag: String = folder_etag.into();
			let mut response = actix_web::HttpResponse::Ok();
			response.content_type("application/ld+json");
			response.header(actix_web::http::header::ETAG, folder_etag);
			response.header(actix_web::http::header::CACHE_CONTROL, "no-cache");
			response.header(actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);

			if origin != "*" {
				response.header(actix_web::http::header::VARY, "Origin");
			}

			return response.finish();
		}
		Ok(pontus_onyx::Item::Folder { content: None, .. }) => {
			pontus_onyx::database::build_http_json_response(
				origin,
				request.method(),
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				false,
			)
		}
		Err(e) => {
			if e.is::<pontus_onyx::database::sources::memory::GetError>() {
				pontus_onyx::database::Error::to_response(
					&*e.downcast::<pontus_onyx::database::sources::memory::GetError>()
						.unwrap(),
					origin,
					true,
				)
			} else if e.is::<pontus_onyx::database::sources::folder::GetError>() {
				pontus_onyx::database::Error::to_response(
					&*e.downcast::<pontus_onyx::database::sources::folder::GetError>()
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
