#[actix_web::delete("/{requested_item:.*}")]
pub async fn delete_item(
	path: actix_web::web::Path<String>,
	request: actix_web::web::HttpRequest,
	database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<pontus_onyx::Database>>>,
) -> actix_web::web::HttpResponse {
	let mut db = database.lock().unwrap();

	let if_match_result = if let Some(find_match) = request.headers().get("If-Match") {
		let find_match = find_match.to_str().unwrap().trim().replace('"', "");

		match db.read(&path) {
			Ok(Some(pontus_onyx::Item::Document {
				etag: document_etag,
				content: _,
				content_type: _,
				last_modified: _,
			})) => Ok(document_etag == find_match),
			Ok(Some(pontus_onyx::Item::Folder {
				etag: _,
				content: _,
			})) => Err(actix_web::HttpResponse::BadRequest()
				.content_type("application/ld+json")
				.body(r#"{"http_code":400,"http_description":"bad request"}"#)),
			Ok(None) => Err(actix_web::HttpResponse::NotFound()
				.content_type("application/ld+json")
				.body(r#"{"http_code":404,"http_description":"requested content not found"}"#)),
			Err(_) => Err(actix_web::HttpResponse::InternalServerError()
				.content_type("application/ld+json")
				.body(r#"{"http_code":500,"http_description":"internal server error"}"#)),
		}
	} else {
		Ok(true)
	};

	match if_match_result {
		Ok(if_match_result) => {
			if if_match_result {
				match db.delete(&path) {
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
					Err(pontus_onyx::DeleteError::DoesNotWorksForFolders) => {
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
							.body(
								r#"{"http_code":404,"http_description":"requested content not found"}"#,
							);
					}
					Err(_todo) => {
						return actix_web::HttpResponse::InternalServerError()
							.content_type("application/ld+json")
							.body(
								r#"{"http_code":500,"http_description":"internal server error"}"#,
							);
					}
				}
			} else {
				return actix_web::HttpResponse::PreconditionFailed().finish();
			}
		}
		Err(e) => {
			return e;
		}
	}
}

mod tests {
	use actix_web::http::{header::EntityTag, Method, StatusCode};

	#[actix_rt::test]
	async fn basics() {
		let database = std::sync::Arc::new(std::sync::Mutex::new(
			pontus_onyx::Database::from_item_folder(pontus_onyx::Item::new_folder(vec![(
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
			)]))
			.unwrap(),
		));

		let mut app = actix_web::test::init_service(
			actix_web::App::new()
				.data(database)
				.service(crate::http_server::api::get_item)
				.service(super::delete_item),
		)
		.await;

		let tests = vec![
			(
				Method::DELETE,
				"/should/not/exists/document",
				StatusCode::NOT_FOUND,
			),
			(
				Method::DELETE,
				"/should/not/exists/folder/",
				StatusCode::BAD_REQUEST,
			),
			(Method::GET, "/a/b/c", StatusCode::OK),
			(Method::DELETE, "/a", StatusCode::NOT_FOUND),
			(Method::DELETE, "/a/", StatusCode::BAD_REQUEST),
			(Method::DELETE, "/a/b", StatusCode::NOT_FOUND),
			(Method::DELETE, "/a/b/", StatusCode::BAD_REQUEST),
			(Method::DELETE, "/a/b/c", StatusCode::OK),
			(Method::GET, "/a/b/c", StatusCode::NOT_FOUND),
			(Method::DELETE, "/a/b/c", StatusCode::NOT_FOUND),
			(Method::GET, "/a/b/", StatusCode::NOT_FOUND),
			(Method::GET, "/a/", StatusCode::NOT_FOUND),
			(Method::GET, "/", StatusCode::OK),
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
	async fn if_match() {
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
				.service(crate::http_server::api::get_item)
				.service(super::delete_item),
		)
		.await;

		let tests = vec![
			(Method::GET, "/a/b/c", vec![], StatusCode::OK),
			(
				Method::DELETE,
				"/a/b/c",
				vec![EntityTag::new(false, String::from("ANOTHER_ETAG"))],
				StatusCode::PRECONDITION_FAILED,
			),
			(Method::GET, "/a/b/c", vec![], StatusCode::OK),
			(Method::GET, "/a/b/c", vec![], StatusCode::OK),
			(
				Method::DELETE,
				"/a/b/c",
				vec![EntityTag::new(false, String::from("A"))],
				StatusCode::OK,
			),
			(Method::GET, "/a/b/c", vec![], StatusCode::NOT_FOUND),
			(
				Method::DELETE,
				"/a/b/c",
				vec![EntityTag::new(false, String::from("A"))],
				StatusCode::NOT_FOUND,
			),
			(
				Method::DELETE,
				"/a/b/c",
				vec![EntityTag::new(false, String::from("ANOTHER_ETAG"))],
				StatusCode::NOT_FOUND,
			),
		];

		for test in tests {
			print!(
				"{} request to {} with If-Match = {:?} : ",
				test.0, test.1, test.2
			);

			let request = actix_web::test::TestRequest::with_uri(test.1)
				.method(test.0)
				.set(actix_web::http::header::IfMatch::Items(test.2))
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), test.3);
			println!("OK");
		}
	}
}
