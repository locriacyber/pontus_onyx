use actix_web::http::{header::EntityTag, Method, StatusCode};

#[actix_rt::test]
async fn basics() {
	let database = pontus_onyx::Database::new(
		&pontus_onyx::database::DataSource::Memory(pontus_onyx::Item::new_folder(vec![
			(
				"user",
				pontus_onyx::Item::new_folder(vec![(
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
				)]),
			),
			(
				"public",
				pontus_onyx::Item::new_folder(vec![(
					"user",
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
				)]),
			),
		])),
		None,
	)
	.unwrap();
	let database = std::sync::Arc::new(std::sync::Mutex::new(database));

	let mut app = actix_web::test::init_service(
		actix_web::App::new()
			.data(database)
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
			StatusCode::NOT_FOUND,
		),
		(060, Method::GET, "/storage/user/a/", StatusCode::OK),
		(070, Method::GET, "/storage/user/a/b/", StatusCode::OK),
		(080, Method::GET, "/storage/user/a/b/c", StatusCode::OK),
		(
			090,
			Method::GET,
			"/storage/public/user",
			StatusCode::CONFLICT,
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
			StatusCode::CONFLICT,
		),
		(
			120,
			Method::GET,
			"/storage/public/user/0/1",
			StatusCode::CONFLICT,
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
	let database = pontus_onyx::Database::new(
		&pontus_onyx::database::DataSource::Memory(pontus_onyx::Item::new_folder(vec![(
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
		)])),
		None,
	)
	.unwrap();
	let database = std::sync::Arc::new(std::sync::Mutex::new(database));

	let mut app = actix_web::test::init_service(
		actix_web::App::new()
			.data(database)
			.service(super::get_item),
	)
	.await;

	let tests = vec![
		(
			010,
			vec![EntityTag::new(false, String::from("A"))],
			StatusCode::PRECONDITION_FAILED,
		),
		(
			020,
			vec![
				EntityTag::new(false, String::from("A")),
				EntityTag::new(false, String::from("B")),
			],
			StatusCode::PRECONDITION_FAILED,
		),
		(
			030,
			vec![EntityTag::new(false, String::from("*"))],
			StatusCode::PRECONDITION_FAILED,
		),
		(
			040,
			vec![EntityTag::new(false, String::from("ANOTHER_ETAG"))],
			StatusCode::OK,
		),
		(
			050,
			vec![
				EntityTag::new(false, String::from("ANOTHER_ETAG_1")),
				EntityTag::new(false, String::from("ANOTHER_ETAG_2")),
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
			.set(actix_web::http::header::IfNoneMatch::Items(test.1.clone()))
			.to_request();
		let response = actix_web::test::call_service(&mut app, request).await;

		assert_eq!(response.status(), test.2);

		println!("OK");
	}
}
