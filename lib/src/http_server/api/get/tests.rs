use actix_web::http::{header::EntityTag, Method, StatusCode};

#[actix_rt::test]
async fn basics() {
	let database =
		crate::database::Database::new(Box::new(crate::database::sources::MemoryStorage {
			root_item: crate::item::Item::new_folder(vec![
				(
					"user",
					crate::item::Item::new_folder(vec![(
						"a",
						crate::item::Item::new_folder(vec![(
							"b",
							crate::item::Item::new_folder(vec![(
								"c",
								crate::item::Item::Document {
									etag: crate::item::Etag::new(),
									content: Some(b"HELLO".to_vec()),
									content_type: crate::item::ContentType::from("text/plain"),
									last_modified: Some(time::OffsetDateTime::now_utc()),
								},
							)]),
						)]),
					)]),
				),
				(
					"public",
					crate::item::Item::new_folder(vec![(
						"user",
						crate::item::Item::new_folder(vec![(
							"0",
							crate::item::Item::new_folder(vec![(
								"1",
								crate::item::Item::new_folder(vec![(
									"2",
									crate::item::Item::Document {
										etag: crate::item::Etag::new(),
										content: Some(b"HELLO".to_vec()),
										content_type: crate::item::ContentType::from("text/plain"),
										last_modified: Some(time::OffsetDateTime::now_utc()),
									},
								)]),
							)]),
						)]),
					)]),
				),
			]),
		}));
	let database = std::sync::Arc::new(std::sync::Mutex::new(database));

	let logger = charlie_buffalo::Logger::new(
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
			.service(super::get_item),
	)
	.await;

	let tests = vec![
		(
			010,
			Method::GET,
			"/storage/user/not/exists/document",
			StatusCode::NOT_FOUND,
		),
		(
			020,
			Method::GET,
			"/storage/user/not/exists/folder/",
			StatusCode::NOT_FOUND,
		),
		(030, Method::GET, "/storage/user/a", StatusCode::CONFLICT),
		(040, Method::GET, "/storage/user/a/b", StatusCode::CONFLICT),
		(
			050,
			Method::GET,
			"/storage/user/a/b/c/",
			StatusCode::CONFLICT,
		),
		(060, Method::GET, "/storage/user/a/", StatusCode::OK),
		(070, Method::GET, "/storage/user/a/b/", StatusCode::OK),
		(080, Method::GET, "/storage/user/a/b/c", StatusCode::OK),
		(
			090,
			Method::GET,
			"/storage/public/user",
			StatusCode::NOT_FOUND,
		),
		(
			100,
			Method::GET,
			"/storage/public/user/",
			StatusCode::NOT_FOUND,
		),
		(
			110,
			Method::GET,
			"/storage/public/user/0",
			StatusCode::NOT_FOUND,
		),
		(
			120,
			Method::GET,
			"/storage/public/user/0/1",
			StatusCode::NOT_FOUND,
		),
		(
			130,
			Method::GET,
			"/storage/public/user/0/1/2",
			StatusCode::OK,
		),
		(
			140,
			Method::GET,
			"/storage/public/user/0/",
			StatusCode::NOT_FOUND,
		),
		(
			150,
			Method::GET,
			"/storage/public/user/0/1/",
			StatusCode::NOT_FOUND,
		),
		(
			160,
			Method::GET,
			"/storage/public/user/0/1/2/",
			StatusCode::NOT_FOUND,
		),
	];

	for test in tests {
		print!("#{:03} : {} request to {} ... ", test.0, test.1, test.2);

		let request = actix_web::test::TestRequest::with_uri(test.2)
			.method(test.1.clone())
			.to_request();
		let response = actix_web::test::call_service(&mut app, request).await;

		assert_eq!(response.status(), test.3);

		println!("OK");
	}
}

#[actix_rt::test]
async fn if_none_match() {
	let database =
		crate::database::Database::new(Box::new(crate::database::sources::MemoryStorage {
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
								content_type: crate::item::ContentType::from("text/plain"),
								last_modified: Some(time::OffsetDateTime::now_utc()),
							},
						)]),
					)]),
				)]),
			)]),
		}));
	let database = std::sync::Arc::new(std::sync::Mutex::new(database));

	let logger = charlie_buffalo::Logger::new(
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
			.service(super::get_item),
	)
	.await;

	let tests = vec![
		(
			010,
			vec![EntityTag::new(false, "A".into())],
			StatusCode::PRECONDITION_FAILED,
		),
		(
			020,
			vec![
				EntityTag::new(false, "A".into()),
				EntityTag::new(false, "B".into()),
			],
			StatusCode::PRECONDITION_FAILED,
		),
		(
			030,
			vec![EntityTag::new(false, "*".into())],
			StatusCode::PRECONDITION_FAILED,
		),
		(
			040,
			vec![EntityTag::new(false, "ANOTHER_ETAG".into())],
			StatusCode::OK,
		),
		(
			050,
			vec![
				EntityTag::new(false, "ANOTHER_ETAG_1".into()),
				EntityTag::new(false, "ANOTHER_ETAG_2".into()),
			],
			StatusCode::OK,
		),
	];

	for test in tests {
		print!(
			"#{:03} : GET request to /storage/user/a/b/c with If-None-Match = {:?} ... ",
			test.0, test.1
		);

		let request = actix_web::test::TestRequest::get()
			.uri("/storage/user/a/b/c")
			.insert_header(actix_web::http::header::IfNoneMatch::Items(test.1.clone()))
			.to_request();
		let response = actix_web::test::call_service(&mut app, request).await;

		assert_eq!(response.status(), test.2);

		println!("OK");
	}
}
