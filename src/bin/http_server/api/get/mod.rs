use std::sync::{Arc, Mutex};

#[actix_web::get("/storage/{requested_item:.*}")]
pub async fn get_item(
	path: actix_web::web::Path<String>,
	request: actix_web::HttpRequest,
	database: actix_web::web::Data<
		std::sync::Arc<std::sync::Mutex<pontus_onyx::database::Database>>,
	>,
	logger: actix_web::web::Data<Arc<Mutex<charlie_buffalo::Logger>>>,
) -> impl actix_web::Responder {
	// TODO : check security issue about this ?
	let all_origins = actix_web::http::header::HeaderValue::from_bytes(b"*").unwrap();
	let origin = request
		.headers()
		.get(actix_web::http::header::ORIGIN)
		.unwrap_or(&all_origins)
		.to_str()
		.unwrap();

	let local_path = pontus_onyx::item::ItemPath::from(path.into_inner().as_str());

	// TODO : If-Match does not works with GET ?
	match database.lock().unwrap().get(
		&local_path,
		super::convert_actix_if_match(&request)
			.first()
			.unwrap_or(&pontus_onyx::item::Etag::from("")),
		&super::convert_actix_if_none_match(&request)
			.iter()
			.collect::<Vec<&pontus_onyx::item::Etag>>(),
	) {
		Ok(pontus_onyx::item::Item::Document {
			etag,
			content: Some(content),
			content_type,
			last_modified,
			..
		}) => {
			let etag: String = etag.into();
			let content_type: String = content_type.into();

			let mut response = actix_web::HttpResponse::Ok();
			response.insert_header((actix_web::http::header::ETAG, etag));
			if let Some(last_modified) = last_modified {
				response.insert_header((
					actix_web::http::header::LAST_MODIFIED,
					last_modified
						.format(&time::format_description::well_known::Rfc2822)
						.unwrap_or_default(),
				));
			}
			response.insert_header((actix_web::http::header::CACHE_CONTROL, "no-cache"));
			response.insert_header((actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin));

			if origin != "*" {
				response.insert_header((actix_web::http::header::VARY, "Origin"));
			}

			response.insert_header((
				actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
				"Content-Length, Content-Type, Etag, Last-Modified",
			));
			response.content_type(content_type);

			return response.body(content);
		}
		Ok(pontus_onyx::item::Item::Folder {
			etag: folder_etag,
			content: Some(content),
		}) => {
			let mut items_result = serde_json::json!({});
			for (child_name, child) in content.iter().filter(|(_, e)| match &***e {
				pontus_onyx::item::Item::Document { .. } => true,
				pontus_onyx::item::Item::Folder {
					content: Some(content),
					..
				} => !content.is_empty(),
				pontus_onyx::item::Item::Folder { content: None, .. } => todo!(),
			}) {
				match &**child {
					pontus_onyx::item::Item::Folder { etag, .. } => {
						items_result[format!("{}/", child_name)] = serde_json::json!({
							"ETag": etag,
						});
					}
					pontus_onyx::item::Item::Document {
						etag,
						content: Some(document_content),
						content_type,
						last_modified,
					} => {
						let child_name: String = child_name.into();
						items_result[child_name] = serde_json::json!({
							"ETag": etag,
							"Content-Type": content_type,
							"Content-Length": document_content.len(),
							"Last-Modified": if let Some(last_modified) = last_modified {
								serde_json::Value::from(last_modified.format(&time::format_description::well_known::Rfc2822).unwrap_or_default())
							} else {
								serde_json::Value::Null
							},
						});
					}
					pontus_onyx::item::Item::Document { content: None, .. } => {
						logger.lock().unwrap().push(
							vec![
								(String::from("level"), String::from("ERROR")),
								(String::from("module"), String::from("https?")),
								(String::from("method"), String::from("GET")),
								(String::from("path"), local_path.to_string()),
							],
							Some("document has no content"),
						);

						return pontus_onyx::database::build_http_json_response(
							origin,
							request.method(),
							actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
							None,
							None,
							None,
							true,
						);
					}
				}
			}

			let folder_etag: String = folder_etag.into();

			let mut response = actix_web::HttpResponse::Ok();
			response.content_type("application/ld+json");
			response.insert_header((actix_web::http::header::ETAG, folder_etag));
			response.insert_header((actix_web::http::header::CACHE_CONTROL, "no-cache"));
			response.insert_header((actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin));

			if origin != "*" {
				response.insert_header((actix_web::http::header::VARY, "Origin"));
			}

			response.insert_header((
				actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
				"Content-Length, Content-Type, Etag, Last-Modified",
			));

			return response.body(
				serde_json::json!({
					"@context": "http://remotestorage.io/spec/folder-description",
					"items": items_result,
				})
				.to_string(),
			);
		}
		Ok(pontus_onyx::item::Item::Document { content: None, .. }) => {
			logger.lock().unwrap().push(
				vec![
					(String::from("level"), String::from("ERROR")),
					(String::from("module"), String::from("https?")),
					(String::from("method"), String::from("GET")),
					(String::from("path"), local_path.to_string()),
				],
				Some("document has no content"),
			);

			return pontus_onyx::database::build_http_json_response(
				origin,
				request.method(),
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				true,
			);
		}
		Ok(pontus_onyx::item::Item::Folder { content: None, .. }) => {
			logger.lock().unwrap().push(
				vec![
					(String::from("level"), String::from("ERROR")),
					(String::from("module"), String::from("https?")),
					(String::from("method"), String::from("GET")),
					(String::from("path"), local_path.to_string()),
				],
				Some("folder has no content"),
			);

			return pontus_onyx::database::build_http_json_response(
				origin,
				request.method(),
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				true,
			);
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
				logger.lock().unwrap().push(
					vec![
						(String::from("level"), String::from("ERROR")),
						(String::from("module"), String::from("https?")),
						(String::from("method"), String::from("GET")),
						(String::from("path"), local_path.to_string()),
					],
					Some(&format!("error from database : {e}")),
				);

				pontus_onyx::database::build_http_json_response(
					origin,
					request.method(),
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
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
