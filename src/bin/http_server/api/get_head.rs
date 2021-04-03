#[actix_web::get("/{requested_item:.*}")]
pub async fn get_item(
	path: actix_web::web::Path<String>,
	request: actix_web::web::HttpRequest,
	database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<pontus_onyx::Database>>>,
) -> actix_web::web::HttpResponse {
	return utils::get(path, request, database, true);
}

#[actix_web::head("/{requested_item:.*}")]
pub async fn head_item(
	path: actix_web::web::Path<String>,
	request: actix_web::web::HttpRequest,
	database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<pontus_onyx::Database>>>,
) -> actix_web::web::HttpResponse {
	return utils::get(path, request, database, false);
}

mod utils {
	pub fn get(
		path: actix_web::web::Path<String>,
		request: actix_web::web::HttpRequest,
		database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<pontus_onyx::Database>>>,
		should_have_body: bool,
	) -> actix_web::web::HttpResponse {
		let should_be_folder = path.split('/').last().unwrap() == "";

		let db = database.lock().unwrap();

		match db.read(&path) {
			Ok(Some(item)) => {
				match item {
					pontus_onyx::Item::Folder {
						etag: folder_etag,
						content,
					} => {
						if should_be_folder {
							if let Some(none_match) = request.headers().get("If-None-Match") {
								let mut none_match = none_match
									.to_str()
									.unwrap()
									.split(',')
									.map(|s| s.trim().replace('"', ""));

								if none_match.any(|s| s == folder_etag || s == "*") {
									return actix_web::HttpResponse::NotModified().finish();
								}
							}

							let mut items_result = serde_json::json!({});
							for (child_name, child) in content.iter().filter(|(_, e)| match &***e {
								pontus_onyx::Item::Document {
									etag: _,
									content: _,
									content_type: _,
									last_modified: _,
								} => true,
								pontus_onyx::Item::Folder { etag: _, content } => {
									!content.is_empty() // TODO : recursive if child is also empty ?
								}
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
											"Last-Modified": last_modified.format(crate::RFC5322).to_string(),
										});
									}
								}
							}

							return actix_web::HttpResponse::Ok()
								.content_type("application/ld+json")
								.header("ETag", folder_etag)
								.header("Cache-Control", "no-cache")
								.body(if should_have_body {
									serde_json::json!({
										"@context": "http://remotestorage.io/spec/folder-description",
										"items": items_result,
									})
									.to_string()
								} else {
									String::new()
								});
						} else {
							// TODO : help user to say there is a folder with this name ?
							return actix_web::HttpResponse::NotFound()
								.content_type("application/ld+json")
								.body(if should_have_body {
									r#"{"http_code":404,"http_description":"requested content not found"}"#
								} else {
									""
								});
						}
					}
					pontus_onyx::Item::Document {
						etag: document_etag,
						content,
						content_type,
						last_modified: _,
					} => {
						if !should_be_folder {
							if let Some(none_match) = request.headers().get("If-None-Match") {
								let mut none_match = none_match
									.to_str()
									.unwrap()
									.split(',')
									.map(|s| s.trim().replace('"', ""));

								if none_match.any(|s| s == document_etag || s == "*") {
									return actix_web::HttpResponse::NotModified().finish();
								}
							}

							return actix_web::HttpResponse::Ok()
								.header("ETag", document_etag)
								.header("Cache-Control", "no-cache")
								.content_type(content_type)
								.body(if should_have_body { content } else { vec![] });
						} else {
							return actix_web::HttpResponse::NotFound()
								.content_type("application/ld+json")
								.body(if should_have_body {
									r#"{"http_code":404,"http_description":"requested content not found"}"#
								} else {
									""
								});
						}
					}
				}
			}
			Ok(None) => {
				return actix_web::HttpResponse::NotFound()
					.content_type("application/ld+json")
					.body(if should_have_body {
						r#"{"http_code":404,"http_description":"requested content not found"}"#
					} else {
						""
					});
			}
			Err(pontus_onyx::ReadError::WrongPath) => {
				return actix_web::HttpResponse::BadRequest()
					.content_type("application/ld+json")
					.body(if should_have_body {
						r#"{"http_code":400,"http_description":"bad request"}"#
					} else {
						""
					});
			}
			Err(err) => {
				println!("ERROR : {:?} : {:?}", path, err); // TODO
				return actix_web::HttpResponse::InternalServerError()
					.content_type("application/ld+json")
					.body(if should_have_body {
						r#"{"http_code":500,"http_description":"internal server error"}"#
					} else {
						""
					});
			}
		};
	}
}
