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
							if path.starts_with("public") {
								return super::super::build_response(
									actix_web::http::StatusCode::NOT_FOUND,
									None,
									None,
									should_have_body,
								);
							} else {
								// TODO : weak headers ?
								if let Some(none_match) = request.headers().get("If-None-Match") {
									let mut none_match = none_match
										.to_str()
										.unwrap()
										.split(',')
										.map(|s| s.trim().replace('"', ""));

									if none_match.any(|s| s == folder_etag || s == "*") {
										return super::super::build_response(
											actix_web::http::StatusCode::NOT_MODIFIED,
											None,
											None,
											should_have_body,
										);
									}
								}

								let mut items_result = serde_json::json!({});
								for (child_name, child) in
									content.iter().filter(|(_, e)| match &***e {
										pontus_onyx::Item::Document {
											etag: _,
											content: _,
											content_type: _,
											last_modified: _,
										} => true,
										pontus_onyx::Item::Folder {
											etag: _,
											content: _,
										} => !e.is_empty(),
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
							}
						} else {
							return super::super::build_response(
								actix_web::http::StatusCode::NOT_FOUND,
								None,
								Some("a folder exists with this name"),
								should_have_body,
							);
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
									return super::super::build_response(
										actix_web::http::StatusCode::NOT_MODIFIED,
										None,
										None,
										should_have_body,
									);
								}
							}

							return actix_web::HttpResponse::Ok()
								.header("ETag", document_etag)
								.header("Cache-Control", "no-cache")
								.content_type(content_type)
								.body(if should_have_body { content } else { vec![] });
						} else {
							return super::super::build_response(
								actix_web::http::StatusCode::NOT_FOUND,
								None,
								None,
								should_have_body,
							);
						}
					}
				}
			}
			Ok(None) => {
				return super::super::build_response(
					actix_web::http::StatusCode::NOT_FOUND,
					None,
					None,
					should_have_body,
				);
			}
			Err(pontus_onyx::ReadError::WrongPath) => {
				return super::super::build_response(
					actix_web::http::StatusCode::BAD_REQUEST,
					None,
					None,
					should_have_body,
				);
			}
			Err(pontus_onyx::ReadError::FolderDocumentConflict) => {
				return super::super::build_response(
					actix_web::http::StatusCode::CONFLICT,
					None,
					None,
					should_have_body,
				);
			}
		};
	}
}

mod tests {
	use actix_web::http::{header::EntityTag, Method, StatusCode};

	#[actix_rt::test]
	async fn basics() {
		let database = std::sync::Arc::new(std::sync::Mutex::new(
			pontus_onyx::Database::from_item_folder(pontus_onyx::Item::new_folder(vec![
				(
					"a",
					pontus_onyx::Item::new_folder(vec![(
						"b",
						pontus_onyx::Item::new_folder(vec![(
							"c",
							pontus_onyx::Item::Document {
								etag: ulid::Ulid::new().to_string(),
								content: b"HELLO".to_vec(),
								content_type: String::from("text/plain"),
								last_modified: chrono::Utc::now(),
							},
						)]),
					)]),
				),
				(
					"public",
					pontus_onyx::Item::new_folder(vec![(
						"0",
						pontus_onyx::Item::new_folder(vec![(
							"1",
							pontus_onyx::Item::new_folder(vec![(
								"2",
								pontus_onyx::Item::Document {
									etag: ulid::Ulid::new().to_string(),
									content: b"HELLO".to_vec(),
									content_type: String::from("text/plain"),
									last_modified: chrono::Utc::now(),
								},
							)]),
						)]),
					)]),
				),
			]))
			.unwrap(),
		));

		let mut app = actix_web::test::init_service(
			actix_web::App::new()
				.data(database)
				.service(super::get_item),
		)
		.await;

		let tests = vec![
			(Method::GET, "/not/exists/document", StatusCode::NOT_FOUND),
			(Method::GET, "/not/exists/folder/", StatusCode::NOT_FOUND),
			(Method::GET, "/a", StatusCode::NOT_FOUND),
			(Method::GET, "/a/b", StatusCode::NOT_FOUND),
			(Method::GET, "/a/b/c/", StatusCode::NOT_FOUND),
			(Method::GET, "/a/", StatusCode::OK),
			(Method::GET, "/a/b/", StatusCode::OK),
			(Method::GET, "/a/b/c", StatusCode::OK),
			(Method::GET, "/public", StatusCode::NOT_FOUND),
			(Method::GET, "/public/", StatusCode::NOT_FOUND),
			(Method::GET, "/public/0", StatusCode::NOT_FOUND),
			(Method::GET, "/public/0/1", StatusCode::NOT_FOUND),
			(Method::GET, "/public/0/1/2", StatusCode::OK),
			(Method::GET, "/public/0/", StatusCode::NOT_FOUND),
			(Method::GET, "/public/0/1/", StatusCode::NOT_FOUND),
			(Method::GET, "/public/0/1/2/", StatusCode::NOT_FOUND),
		];

		for test in tests {
			print!("{} request to {} : ", test.0, test.1);

			let request = actix_web::test::TestRequest::with_uri(test.1)
				.method(test.0)
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), test.2);
			println!("OK");
		}
	}

	#[actix_rt::test]
	async fn if_none_match() {
		let database = std::sync::Arc::new(std::sync::Mutex::new(
			pontus_onyx::Database::from_item_folder(pontus_onyx::Item::new_folder(vec![(
				"a",
				pontus_onyx::Item::new_folder(vec![(
					"b",
					pontus_onyx::Item::new_folder(vec![(
						"c",
						pontus_onyx::Item::Document {
							etag: String::from("A"),
							content: b"HELLO".to_vec(),
							content_type: String::from("text/plain"),
							last_modified: chrono::Utc::now(),
						},
					)]),
				)]),
			)]))
			.unwrap(),
		));

		let mut app = actix_web::test::init_service(
			actix_web::App::new()
				.data(database)
				.service(super::get_item),
		)
		.await;

		let tests = vec![
			(
				vec![EntityTag::new(false, String::from("A"))],
				StatusCode::NOT_MODIFIED,
			),
			(
				vec![
					EntityTag::new(false, String::from("A")),
					EntityTag::new(false, String::from("B")),
				],
				StatusCode::NOT_MODIFIED,
			),
			(
				vec![EntityTag::new(false, String::from("*"))],
				StatusCode::NOT_MODIFIED,
			),
			(
				vec![EntityTag::new(false, String::from("ANOTHER_ETAG"))],
				StatusCode::OK,
			),
			(
				vec![
					EntityTag::new(false, String::from("ANOTHER_ETAG_1")),
					EntityTag::new(false, String::from("ANOTHER_ETAG_2")),
				],
				StatusCode::OK,
			),
		];

		for test in tests {
			print!("GET request to /a/b/c with If-None-Match = {:?} : ", test.0);

			let request = actix_web::test::TestRequest::get()
				.uri("/a/b/c")
				.set(actix_web::http::header::IfNoneMatch::Items(test.0))
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), test.1);
			println!("OK");
		}
	}
}
