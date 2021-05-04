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
) -> Result<actix_web::web::HttpResponse, actix_web::Error> {
	let mut body = actix_web::web::BytesMut::new();
	while let Some(content) = futures::StreamExt::next(&mut request_payload).await {
		let content = content?;
		body.extend_from_slice(&content);
	}
	let body = body.freeze();

	let content_type = request.headers().get("content-type");

	if content_type.is_none() {
		return Ok(super::build_response(
			actix_web::http::StatusCode::BAD_REQUEST,
			None,
			Some("missing content-type HTTP header"),
			true,
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
				return Ok(super::build_response(
					actix_web::http::StatusCode::PRECONDITION_FAILED,
					Some(document_etag.clone()),
					None,
					true,
				));
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
			return Ok(super::build_response(
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				None,
				true,
			));
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
				return Ok(super::build_response(
					actix_web::http::StatusCode::OK,
					Some(new_etag),
					None,
					true,
				));
			}
			Err(pontus_onyx::UpdateError::WrongPath) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::BAD_REQUEST,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::UpdateError::FolderDocumentConflict) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::CONFLICT,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::UpdateError::NotFound) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::NOT_FOUND,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::UpdateError::NotModified) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::NOT_MODIFIED,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::UpdateError::DoesNotWorksForFolders) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::BAD_REQUEST,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::UpdateError::InternalError) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::UpdateError::UpdateFoldersEtagsError(
				pontus_onyx::UpdateFoldersEtagsError::FolderDocumentConflict,
			)) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::CONFLICT,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::UpdateError::UpdateFoldersEtagsError(
				pontus_onyx::UpdateFoldersEtagsError::MissingFolder,
			)) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::UpdateError::UpdateFoldersEtagsError(
				pontus_onyx::UpdateFoldersEtagsError::WrongFolderName,
			)) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::BAD_REQUEST,
					None,
					None,
					true,
				));
			}
		}
	} else {
		match db.create(&path, &body, actix_web::HttpMessage::content_type(&request)) {
			Ok(new_etag) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::CREATED,
					Some(new_etag),
					None,
					true,
				));
			}
			Err(pontus_onyx::CreateError::AlreadyExists) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::PRECONDITION_FAILED,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::CreateError::WrongPath) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::BAD_REQUEST,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::CreateError::FolderDocumentConflict) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::CONFLICT,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::CreateError::NotFound) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::NOT_FOUND,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::CreateError::InternalError) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::CreateError::DoesNotWorksForFolders) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::BAD_REQUEST,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::CreateError::ShouldBeFolder) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::BAD_REQUEST,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::CreateError::FolderBuildError(
				pontus_onyx::FolderBuildError::FolderDocumentConflict,
			)) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::CONFLICT,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::CreateError::FolderBuildError(
				pontus_onyx::FolderBuildError::WrongFolderName,
			)) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::BAD_REQUEST,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::CreateError::UpdateFoldersEtagsError(
				pontus_onyx::UpdateFoldersEtagsError::FolderDocumentConflict,
			)) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::CONFLICT,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::CreateError::UpdateFoldersEtagsError(
				pontus_onyx::UpdateFoldersEtagsError::MissingFolder,
			)) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
					None,
					true,
				));
			}
			Err(pontus_onyx::CreateError::UpdateFoldersEtagsError(
				pontus_onyx::UpdateFoldersEtagsError::WrongFolderName,
			)) => {
				return Ok(super::build_response(
					actix_web::http::StatusCode::BAD_REQUEST,
					None,
					None,
					true,
				));
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use actix_web::http::{header::EntityTag, StatusCode};

	#[actix_rt::test]
	async fn basics() {
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
		let database = std::sync::Arc::new(std::sync::Mutex::new(
			pontus_onyx::Database::from_item_folder(pontus_onyx::Item::new_folder(vec![(
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
			)]))
			.unwrap(),
		));

		let mut app = actix_web::test::init_service(
			actix_web::App::new()
				.data(database.clone())
				.service(super::put_item),
		)
		.await;

		let tests = vec![
			(
				"/storage/user/a/b/c",
				vec![EntityTag::new(false, String::from("A"))],
				StatusCode::PRECONDITION_FAILED,
			),
			(
				"/storage/user/a/b/c",
				vec![
					EntityTag::new(false, String::from("A")),
					EntityTag::new(false, String::from("B")),
				],
				StatusCode::PRECONDITION_FAILED,
			),
			(
				"/storage/user/a/b/c",
				vec![EntityTag::new(false, String::from("*"))],
				StatusCode::PRECONDITION_FAILED,
			),
			(
				"/storage/user/a/b/c",
				vec![EntityTag::new(false, String::from("ANOTHER_ETAG"))],
				StatusCode::OK,
			),
			(
				"/storage/user/a/b/d",
				vec![
					EntityTag::new(false, String::from("ANOTHER_ETAG_1")),
					EntityTag::new(false, String::from("ANOTHER_ETAG_2")),
				],
				StatusCode::OK,
			),
			(
				"/storage/user/new/a",
				vec![EntityTag::new(false, String::from("*"))],
				StatusCode::CREATED,
			),
			(
				"/storage/user/new/a",
				vec![EntityTag::new(false, String::from("*"))],
				StatusCode::PRECONDITION_FAILED,
			),
		];

		for test in tests {
			print!(
				"PUT request to {} with If-None-Math = {:?} ... ",
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

	#[actix_rt::test]
	async fn if_match() {
		let database = std::sync::Arc::new(std::sync::Mutex::new(
			pontus_onyx::Database::from_item_folder(pontus_onyx::Item::new_folder(vec![(
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
			)]))
			.unwrap(),
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
