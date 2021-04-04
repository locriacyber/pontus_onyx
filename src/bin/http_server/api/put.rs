/*
TODO :
	A request is considered successful if the HTTP response code is in
	the 2xx range (e.g. 200 OK, 201 Created), and unsuccessful if an
	error occurred or a condition was not met, e.g. response code 404
	Not Found, 304 Not Modified.
*/
/*
TODO :
	PUT and DELETE requests only need to be made to documents, and never
	to folders. A document PUT will make all ancestor folders along its
	path become non-empty; deleting the last document from a subtree
	will make that whole subtree become empty. Folders will therefore
	show up in their parent folder descriptions if and only if their
	subtree contains at least one document.
*/
/*
TODO :
	Unless [KERBEROS] is used (see section 10 below), all other
	requests SHOULD present a bearer token with sufficient access scope,
	using a header of the following form (no double quotes here):
		Authorization: Bearer <access_token>
*/
#[actix_web::put("/{requested_item:.*}")]
pub async fn put_item(
	mut request_payload: actix_web::web::Payload,
	request: actix_web::web::HttpRequest,
	path: actix_web::web::Path<String>,
	database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<pontus_onyx::Database>>>,
) -> Result<actix_web::web::HttpResponse, actix_web::Error> {
	let mut body = actix_web::web::BytesMut::new();
	while let Some(content) = futures::StreamExt::next(&mut request_payload).await {
		let content = content?;
		body.extend_from_slice(&content);
	}
	let body = body.freeze();

	let content_type = request.headers().get("content-type");

	if content_type.is_none() {
		return Ok(actix_web::HttpResponse::BadRequest()
			.content_type("application/ld+json")
			.body(
				r#"{"http_code":400,"http_description":"bad request","hint":"missing content-type HTTP header"}"#,
			));
	}

	let mut db = database.lock().unwrap();

	let found = db.read(&path)?;
	if let Some(none_match) = request.headers().get("If-None-Match") {
		let mut none_match = none_match
			.to_str()
			.unwrap()
			.split(',')
			.map(|s| s.trim().replace('"', ""));

		if let Some(pontus_onyx::Item::Document {
			etag: document_etag,
			content: _,
			content_type: _,
			last_modified: _,
		}) = &found
		{
			if none_match.any(|s| &s == document_etag || s == "*") {
				return Ok(actix_web::HttpResponse::PreconditionFailed().finish());
			}
		}
	}

	/*
	TODO :
		* its version being updated, as well as that of its parent folder
			and further ancestor folders, using a strong validator [HTTP,
			section 7.2].
	*/
	if found.is_some() {
		let if_match_result = if let Some(find_match) = request.headers().get("If-Match") {
			let find_match = find_match.to_str().unwrap().trim().replace('"', "");

			if let Some(pontus_onyx::Item::Document {
				etag: document_etag,
				content: _,
				content_type: _,
				last_modified: _,
			}) = &found
			{
				document_etag == &find_match
			} else {
				true
			}
		} else {
			true
		};

		if !if_match_result {
			return Ok(actix_web::HttpResponse::PreconditionFailed().finish());
		}

		match db.update(
			&path,
			pontus_onyx::Item::Document {
				etag: ulid::Ulid::new().to_string(),
				content: body.to_vec(),
				content_type: String::from(actix_web::HttpMessage::content_type(&request)),
				last_modified: chrono::Utc::now(),
			},
		) {
			Ok(new_etag) => {
				return Ok(actix_web::HttpResponse::Ok()
					.content_type("application/ld+json")
					.header("ETag", new_etag.clone())
					.body(format!(
						r#"{{"http_code":200,"http_description":"success","ETag":"{}"}}"#,
						new_etag
					)))
			}
			Err(pontus_onyx::UpdateError::WrongPath) => {
				return Ok(actix_web::HttpResponse::BadRequest()
					.content_type("application/ld+json")
					.body(r#"{"http_code":400,"http_description":"bad request"}"#))
			}
			Err(pontus_onyx::UpdateError::FolderDocumentConflict) => {
				return Ok(actix_web::HttpResponse::Conflict()
					.content_type("application/ld+json")
					.body(r#"{"http_code":409,"http_description":"conflict"}"#))
			}
			Err(pontus_onyx::UpdateError::NotFound) => {
				return Ok(actix_web::HttpResponse::NotFound()
					.content_type("application/ld+json")
					.body(r#"{"http_code":404,"http_description":"requested content not found"}"#))
			}
			Err(_todo) => {
				return Ok(actix_web::HttpResponse::InternalServerError()
					.content_type("application/ld+json")
					.body(r#"{"http_code":500,"http_description":"internal server error"}"#))
			}
		}
	} else {
		match db.create(&path, &body, actix_web::HttpMessage::content_type(&request)) {
			Ok(new_etag) => {
				return Ok(actix_web::HttpResponse::Created()
					.content_type("application/ld+json")
					.header("ETag", new_etag.clone())
					.body(format!(
						r#"{{"http_code":201,"http_description":"created","ETag":"{}"}}"#,
						new_etag
					)));
			}
			Err(pontus_onyx::CreateError::AlreadyExists) => {
				return Ok(actix_web::HttpResponse::PreconditionFailed()
					.content_type("application/ld+json")
					.body(r#"{{"http_code":412,"http_description":"precondition failed"}}"#));
			}
			Err(pontus_onyx::CreateError::WrongPath) => {
				return Ok(actix_web::HttpResponse::BadRequest()
					.content_type("application/ld+json")
					.body(r#"{"http_code":400,"http_description":"bad request"}"#));
			}
			Err(pontus_onyx::CreateError::FolderDocumentConflict) => {
				return Ok(actix_web::HttpResponse::Conflict()
					.content_type("application/ld+json")
					.body(r#"{"http_code":409,"http_description":"conflict"}"#));
			}
			Err(pontus_onyx::CreateError::NotFound) => {
				return Ok(actix_web::HttpResponse::NotFound()
					.content_type("application/ld+json")
					.body(
						r#"{"http_code":404,"http_description":"requested content not found"}"#,
					));
			}
			Err(_todo) => {
				return Ok(actix_web::HttpResponse::InternalServerError()
					.content_type("application/ld+json")
					.body(r#"{"http_code":500,"http_description":"internal server error"}"#));
			}
		}
	}
}

mod tests {
	use actix_web::http::{header::EntityTag, StatusCode};

	#[actix_rt::test]
	async fn k7ip00aqdgtrjdt() {
		let database = std::sync::Arc::new(std::sync::Mutex::new(
			pontus_onyx::Database::from_item_folder(pontus_onyx::Item::new_folder(vec![])).unwrap(),
		));

		let mut app = actix_web::test::init_service(
			actix_web::App::new()
				.data(database)
				.service(crate::http_server::api::get_item)
				.service(super::put_item),
		)
		.await;

		{
			let request = actix_web::test::TestRequest::get()
				.uri("/a/b/c")
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), StatusCode::NOT_FOUND);
		}

		{
			let request = actix_web::test::TestRequest::put()
				.uri("/a/b/c")
				.set(actix_web::http::header::ContentType::plaintext())
				.set_payload(b"EVERYONE".to_vec())
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), StatusCode::CREATED);
		}

		{
			let request = actix_web::test::TestRequest::get()
				.uri("/a/b/c")
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), StatusCode::OK);
		}

		{
			let request = actix_web::test::TestRequest::put()
				.uri("/a/b/c")
				.set(actix_web::http::header::ContentType::plaintext())
				.set_payload(b"SOMEONE HERE ?".to_vec())
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), StatusCode::OK);
		}

		{
			let request = actix_web::test::TestRequest::get()
				.uri("/a/b/c")
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), StatusCode::OK);
		}
	}

	#[actix_rt::test]
	async fn ddj0vihc3rvc0zd() {
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
				.service(super::put_item),
		)
		.await;

		let tests = vec![
			(
				"/a/b/c",
				vec![EntityTag::new(false, String::from("A"))],
				StatusCode::PRECONDITION_FAILED,
			),
			(
				"/a/b/c",
				vec![
					EntityTag::new(false, String::from("A")),
					EntityTag::new(false, String::from("B")),
				],
				StatusCode::PRECONDITION_FAILED,
			),
			(
				"/a/b/c",
				vec![EntityTag::new(false, String::from("*"))],
				StatusCode::PRECONDITION_FAILED,
			),
			(
				"/a/b/c",
				vec![EntityTag::new(false, String::from("ANOTHER_ETAG"))],
				StatusCode::OK,
			),
			(
				"/a/b/c",
				vec![
					EntityTag::new(false, String::from("ANOTHER_ETAG_1")),
					EntityTag::new(false, String::from("ANOTHER_ETAG_2")),
				],
				StatusCode::OK,
			),
			(
				"/new/a",
				vec![EntityTag::new(false, String::from("*"))],
				StatusCode::CREATED,
			),
			(
				"/new/a",
				vec![EntityTag::new(false, String::from("*"))],
				StatusCode::PRECONDITION_FAILED,
			),
		];

		for test in tests {
			print!(
				"PUT request to {} with If-None-Math = {:?} : ",
				test.0, test.1
			);

			let request = actix_web::test::TestRequest::put()
				.uri(test.0)
				.set(actix_web::http::header::IfNoneMatch::Items(test.1))
				.set_json(&serde_json::json!({"value": "C"}))
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), test.2);
			println!("OK");
		}
	}

	// TODO : test If-Match here
}
