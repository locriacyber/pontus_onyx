/*
TODO :
	Unless [KERBEROS] is used (see section 10 below), all other
	requests SHOULD present a bearer token with sufficient access scope,
	using a header of the following form (no double quotes here):
		Authorization: Bearer <access_token>
*/
#[actix_web::put("/storage/{requested_item:.*}")]
pub async fn put_item(
	mut request_payload: actix_web::web::Payload,
	request: actix_web::web::HttpRequest,
	path: actix_web::web::Path<String>,
	database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<pontus_onyx::Database>>>,
) -> actix_web::web::HttpResponse {
	let mut content = actix_web::web::BytesMut::new();
	while let Some(request_body) = futures::StreamExt::next(&mut request_payload).await {
		let request_body = request_body.unwrap();
		content.extend_from_slice(&request_body);
	}
	let content = content.freeze();

	let content_type = request.headers().get("content-type");

	if content_type.is_none() {
		return pontus_onyx::build_http_json_response(
			request.method(),
			actix_web::http::StatusCode::BAD_REQUEST,
			None,
			Some(String::from("missing content-type HTTP header")),
			true,
		);
	}

	let if_none_match = request
		.headers()
		.get("If-None-Match")
		.map(|e| (e.to_str().unwrap()).split(',').collect::<Vec<&str>>());

	match database.lock().unwrap().put(
		&path,
		pontus_onyx::Item::Document {
			etag: String::new(),
			content: content.to_vec(),
			content_type: String::from(content_type.unwrap().to_str().unwrap()),
			last_modified: chrono::Utc::now(),
		},
		request
			.headers()
			.get("If-Match")
			.map(|e| e.to_str().unwrap()),
		if_none_match,
	) {
		pontus_onyx::database::ResultPut::Created(new_etag) => {
			return pontus_onyx::build_http_json_response(
				request.method(),
				actix_web::http::StatusCode::CREATED,
				Some(new_etag),
				None,
				true,
			);
		}
		pontus_onyx::database::ResultPut::Updated(new_etag) => {
			return pontus_onyx::build_http_json_response(
				request.method(),
				actix_web::http::StatusCode::OK,
				Some(new_etag),
				None,
				true,
			);
		}
		pontus_onyx::database::ResultPut::Err(e) => actix_web::HttpResponse::from(e),
	}
}

#[cfg(test)]
mod tests {
	use actix_web::http::{header::EntityTag, StatusCode};

	#[actix_rt::test]
	async fn basics() {
		let (database, _) = pontus_onyx::Database::new(pontus_onyx::database::DataSource::Memory(
			pontus_onyx::Item::new_folder(vec![]),
		))
		.unwrap();
		let database = std::sync::Arc::new(std::sync::Mutex::new(database));

		let mut app = actix_web::test::init_service(
			actix_web::App::new()
				.data(database)
				.service(crate::http_server::api::get_item)
				.service(super::put_item),
		)
		.await;

		{
			let request = actix_web::test::TestRequest::get()
				.uri("/storage/user/a/b/c")
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), StatusCode::NOT_FOUND);
		}

		{
			let request = actix_web::test::TestRequest::put()
				.uri("/storage/user/a/b/c")
				.set(actix_web::http::header::ContentType::plaintext())
				.set_payload(b"EVERYONE".to_vec())
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), StatusCode::CREATED);
		}

		{
			let request = actix_web::test::TestRequest::get()
				.uri("/storage/user/a/b/c")
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), StatusCode::OK);
		}

		{
			let request = actix_web::test::TestRequest::put()
				.uri("/storage/user/a/b/c")
				.set(actix_web::http::header::ContentType::plaintext())
				.set_payload(b"SOMEONE HERE ?".to_vec())
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), StatusCode::OK);
		}

		{
			let request = actix_web::test::TestRequest::get()
				.uri("/storage/user/a/b/c")
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), StatusCode::OK);
		}

		{
			let request = actix_web::test::TestRequest::put()
				.uri("/storage/user/a/b/c")
				.set(actix_web::http::header::ContentType::plaintext())
				.set_payload(b"SOMEONE HERE ?".to_vec())
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
		}
	}

	#[actix_rt::test]
	async fn if_none_match() {
		let (database, _) = pontus_onyx::Database::new(pontus_onyx::database::DataSource::Memory(
			pontus_onyx::Item::new_folder(vec![(
				"user",
				pontus_onyx::Item::new_folder(vec![(
					"a",
					pontus_onyx::Item::new_folder(vec![(
						"b",
						pontus_onyx::Item::new_folder(vec![
							(
								"c",
								pontus_onyx::Item::Document {
									etag: String::from("A"),
									content: b"HELLO".to_vec(),
									content_type: String::from("text/plain"),
									last_modified: chrono::Utc::now(),
								},
							),
							(
								"d",
								pontus_onyx::Item::Document {
									etag: String::from("A"),
									content: b"HELLO".to_vec(),
									content_type: String::from("text/plain"),
									last_modified: chrono::Utc::now(),
								},
							),
						]),
					)]),
				)]),
			)]),
		))
		.unwrap();
		let database = std::sync::Arc::new(std::sync::Mutex::new(database));

		let mut app = actix_web::test::init_service(
			actix_web::App::new()
				.data(database.clone())
				.service(super::put_item),
		)
		.await;

		let tests = vec![
			(
				010,
				"/storage/user/a/b/c",
				vec![EntityTag::new(false, String::from("A"))],
				StatusCode::PRECONDITION_FAILED,
			),
			(
				020,
				"/storage/user/a/b/c",
				vec![
					EntityTag::new(false, String::from("A")),
					EntityTag::new(false, String::from("B")),
				],
				StatusCode::PRECONDITION_FAILED,
			),
			(
				030,
				"/storage/user/a/b/c",
				vec![EntityTag::new(false, String::from("*"))],
				StatusCode::PRECONDITION_FAILED,
			),
			(
				040,
				"/storage/user/a/b/c",
				vec![EntityTag::new(false, String::from("ANOTHER_ETAG"))],
				StatusCode::OK,
			),
			(
				050,
				"/storage/user/a/b/d",
				vec![
					EntityTag::new(false, String::from("ANOTHER_ETAG_1")),
					EntityTag::new(false, String::from("ANOTHER_ETAG_2")),
				],
				StatusCode::OK,
			),
			(
				060,
				"/storage/user/new/a",
				vec![EntityTag::new(false, String::from("*"))],
				StatusCode::CREATED,
			),
			(
				070,
				"/storage/user/new/a",
				vec![EntityTag::new(false, String::from("*"))],
				StatusCode::PRECONDITION_FAILED,
			),
		];

		for test in tests {
			print!(
				"#{:03} : PUT request to {} with If-None-Math = {:?} ... ",
				test.0, test.1, test.2
			);

			let request = actix_web::test::TestRequest::put()
				.uri(test.1)
				.set(actix_web::http::header::IfNoneMatch::Items(test.2.clone()))
				.set_json(&serde_json::json!({"value": "C"}))
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), test.3);

			println!("OK");
		}
	}

	#[actix_rt::test]
	async fn if_match() {
		let (database, _) = pontus_onyx::Database::new(pontus_onyx::database::DataSource::Memory(
			pontus_onyx::Item::new_folder(vec![(
				"user",
				pontus_onyx::Item::new_folder(vec![(
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
				)]),
			)]),
		))
		.unwrap();
		let database = std::sync::Arc::new(std::sync::Mutex::new(database));

		let mut app = actix_web::test::init_service(
			actix_web::App::new()
				.data(database)
				.service(crate::http_server::api::get_item)
				.service(super::put_item),
		)
		.await;

		{
			let request = actix_web::test::TestRequest::get()
				.uri("/storage/user/a/b/c")
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), StatusCode::OK);
		}

		{
			let request = actix_web::test::TestRequest::put()
				.uri("/storage/user/a/b/c")
				.set(actix_web::http::header::IfMatch::Items(vec![
					EntityTag::new(false, String::from("ANOTHER_ETAG")),
				]))
				.set_json(&serde_json::json!({"value": "C"}))
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), StatusCode::PRECONDITION_FAILED);
		}

		{
			let request = actix_web::test::TestRequest::put()
				.uri("/storage/user/a/b/c")
				.set(actix_web::http::header::IfMatch::Items(vec![
					EntityTag::new(false, String::from("A")),
				]))
				.set_json(&serde_json::json!({"value": "C"}))
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), StatusCode::OK);
		}
	}
}
