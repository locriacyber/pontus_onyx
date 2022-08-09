use actix_web::http::{header::EntityTag, StatusCode};

#[actix_rt::test]
async fn basics() {
	let database = crate::database::Database::new(Box::new(
		crate::database::sources::MemoryStorage {
			root_item: crate::item::Item::new_folder(vec![]),
		},
	));
	let database = std::sync::Arc::new(std::sync::Mutex::new(database));

	let mut logger = charlie_buffalo::Logger::new(
		charlie_buffalo::new_dispatcher(Box::from(move |log: charlie_buffalo::Log| {
			println!("{:?} : {:?}", log.attributes, log.content);
		})),
		charlie_buffalo::new_dropper(Box::from(|_: &charlie_buffalo::Logger| {})),
	);
	let logger = std::sync::Arc::new(std::sync::Mutex::new(logger));

	let mut app = actix_web::test::init_service(
		actix_web::App::new()
			.app_data(actix_web::web::Data::new(database))
			.app_data(actix_web::web::Data::new(logger))
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
			.insert_header(actix_web::http::header::ContentType::plaintext())
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
			.insert_header(actix_web::http::header::ContentType::plaintext())
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
			.insert_header(actix_web::http::header::ContentType::plaintext())
			.set_payload(b"SOMEONE HERE ?".to_vec())
			.to_request();
		let response = actix_web::test::call_service(&mut app, request).await;

		assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
	}
}

#[actix_rt::test]
async fn if_none_match() {
	let database = crate::database::Database::new(Box::new(
		crate::database::sources::MemoryStorage {
			root_item: crate::item::Item::new_folder(vec![(
				"user",
				crate::item::Item::new_folder(vec![(
					"a",
					crate::item::Item::new_folder(vec![(
						"b",
						crate::item::Item::new_folder(vec![
							(
								"c",
								crate::item::Item::Document {
									etag: crate::item::Etag::from("A"),
									content: Some(b"HELLO".to_vec()),
									content_type: crate::item::ContentType::from(
										"text/plain",
									),
									last_modified: Some(time::OffsetDateTime::now_utc()),
								},
							),
							(
								"d",
								crate::item::Item::Document {
									etag: crate::item::Etag::from("A"),
									content: Some(b"HELLO".to_vec()),
									content_type: crate::item::ContentType::from(
										"text/plain",
									),
									last_modified: Some(time::OffsetDateTime::now_utc()),
								},
							),
						]),
					)]),
				)]),
			)]),
		},
	));
	let database = std::sync::Arc::new(std::sync::Mutex::new(database));

	let mut logger = charlie_buffalo::Logger::new(
		charlie_buffalo::new_dispatcher(Box::from(move |log: charlie_buffalo::Log| {
			println!("{:?} : {:?}", log.attributes, log.content);
		})),
		charlie_buffalo::new_dropper(Box::from(|_: &charlie_buffalo::Logger| {})),
	);
	let logger = std::sync::Arc::new(std::sync::Mutex::new(logger));

	let mut app = actix_web::test::init_service(
		actix_web::App::new()
			.app_data(actix_web::web::Data::new(database.clone()))
			.app_data(actix_web::web::Data::new(logger.clone()))
			.service(super::put_item),
	)
	.await;

	let tests = vec![
		(
			010,
			"/storage/user/a/b/c",
			vec![EntityTag::new(false, "A".into())],
			StatusCode::PRECONDITION_FAILED,
		),
		(
			020,
			"/storage/user/a/b/c",
			vec![
				EntityTag::new(false, "A".into()),
				EntityTag::new(false, "B".into()),
			],
			StatusCode::PRECONDITION_FAILED,
		),
		(
			030,
			"/storage/user/a/b/c",
			vec![EntityTag::new(false, "*".into())],
			StatusCode::PRECONDITION_FAILED,
		),
		(
			040,
			"/storage/user/a/b/c",
			vec![EntityTag::new(false, "ANOTHER_ETAG".into())],
			StatusCode::OK,
		),
		(
			050,
			"/storage/user/a/b/d",
			vec![
				EntityTag::new(false, "ANOTHER_ETAG_1".into()),
				EntityTag::new(false, "ANOTHER_ETAG_2".into()),
			],
			StatusCode::OK,
		),
		(
			060,
			"/storage/user/new/a",
			vec![EntityTag::new(false, "*".into())],
			StatusCode::CREATED,
		),
		(
			070,
			"/storage/user/new/a",
			vec![EntityTag::new(false, "*".into())],
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
			.insert_header(actix_web::http::header::IfNoneMatch::Items(test.2.clone()))
			.set_json(&serde_json::json!({"value": "C"}))
			.to_request();
		let response = actix_web::test::call_service(&mut app, request).await;

		assert_eq!(response.status(), test.3);

		println!("OK");
	}
}

#[actix_rt::test]
async fn if_match() {
	let database = crate::database::Database::new(Box::new(
		crate::database::sources::MemoryStorage {
			root_item: crate::item::Item::new_folder(vec![(
				"user",
				crate::item::Item::new_folder(vec![(
					"a",
					crate::item::Item::new_folder(vec![(
						"b",
						crate::item::Item::new_folder(vec![(
							"c",
							crate::item::Item::Document {
								etag: crate::item::Etag::from("A"),
								content: Some(b"HELLO".to_vec()),
								content_type: crate::item::ContentType::from(
									"text/plain",
								),
								last_modified: Some(time::OffsetDateTime::now_utc()),
							},
						)]),
					)]),
				)]),
			)]),
		},
	));
	let database = std::sync::Arc::new(std::sync::Mutex::new(database));

	let mut logger = charlie_buffalo::Logger::new(
		charlie_buffalo::new_dispatcher(Box::from(move |log: charlie_buffalo::Log| {
			println!("{:?} : {:?}", log.attributes, log.content);
		})),
		charlie_buffalo::new_dropper(Box::from(|_: &charlie_buffalo::Logger| {})),
	);
	let logger = std::sync::Arc::new(std::sync::Mutex::new(logger));

	let mut app = actix_web::test::init_service(
		actix_web::App::new()
			.app_data(actix_web::web::Data::new(database))
			.app_data(actix_web::web::Data::new(logger))
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
			.insert_header(actix_web::http::header::IfMatch::Items(vec![
				EntityTag::new(false, "ANOTHER_ETAG".into()),
			]))
			.set_json(&serde_json::json!({"value": "C"}))
			.to_request();
		let response = actix_web::test::call_service(&mut app, request).await;

		assert_eq!(response.status(), StatusCode::PRECONDITION_FAILED);
	}

	{
		let request = actix_web::test::TestRequest::put()
			.uri("/storage/user/a/b/c")
			.insert_header(actix_web::http::header::IfMatch::Items(vec![
				EntityTag::new(false, "A".into()),
			]))
			.set_json(&serde_json::json!({"value": "C"}))
			.to_request();
		let response = actix_web::test::call_service(&mut app, request).await;

		assert_eq!(response.status(), StatusCode::OK);
	}
}
