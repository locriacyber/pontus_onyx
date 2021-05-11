#[actix_web::delete("/storage/{requested_item:.*}")]
pub async fn delete_item(
	path: actix_web::web::Path<String>,
	request: actix_web::web::HttpRequest,
	database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<pontus_onyx::Database>>>,
) -> actix_web::web::HttpResponse {
	match database.lock().unwrap().delete(
		&path,
		request
			.headers()
			.get("If-Match")
			.map(|e| e.to_str().unwrap()),
	) {
		Ok(etag) => {
			return pontus_onyx::build_http_json_response(
				actix_web::http::StatusCode::OK,
				Some(etag),
				None,
				true,
			);
		}
		Err(e) => e.into(),
	}
}

#[cfg(test)]
mod tests {
	use actix_web::http::{header::EntityTag, Method, StatusCode};

	#[actix_rt::test]
	async fn basics() {
		let database = std::sync::Arc::new(std::sync::Mutex::new(
			pontus_onyx::Database::new(pontus_onyx::Source::Memory(pontus_onyx::Item::new_folder(
				vec![(
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
				)],
			)))
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
				010,
				Method::DELETE,
				"/storage/user/should/not/exists/document",
				StatusCode::NOT_FOUND,
			),
			(
				020,
				Method::DELETE,
				"/storage/user/should/not/exists/folder/",
				StatusCode::BAD_REQUEST,
			),
			(030, Method::GET, "/storage/user/a/b/c", StatusCode::OK),
			(040, Method::DELETE, "/storage/user/a", StatusCode::CONFLICT),
			(
				050,
				Method::DELETE,
				"/storage/user/a/",
				StatusCode::BAD_REQUEST,
			),
			(
				060,
				Method::DELETE,
				"/storage/user/a/b",
				StatusCode::CONFLICT,
			),
			(
				070,
				Method::DELETE,
				"/storage/user/a/b/",
				StatusCode::BAD_REQUEST,
			),
			(080, Method::DELETE, "/storage/user/a/b/c", StatusCode::OK),
			(
				090,
				Method::GET,
				"/storage/user/a/b/c",
				StatusCode::NOT_FOUND,
			),
			(
				100,
				Method::DELETE,
				"/storage/user/a/b/c",
				StatusCode::NOT_FOUND,
			),
			(
				110,
				Method::GET,
				"/storage/user/a/b/",
				StatusCode::NOT_FOUND,
			),
			(120, Method::GET, "/storage/user/a/", StatusCode::NOT_FOUND),
			(130, Method::GET, "/storage/user/", StatusCode::NOT_FOUND),
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
	async fn if_match() {
		let database = std::sync::Arc::new(std::sync::Mutex::new(
			pontus_onyx::Database::new(pontus_onyx::Source::Memory(pontus_onyx::Item::new_folder(
				vec![(
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
				)],
			)))
			.unwrap(),
		));

		let mut app = actix_web::test::init_service(
			actix_web::App::new()
				.data(database)
				.service(crate::http_server::api::get_item)
				.service(super::delete_item),
		)
		.await;

		let tests: Vec<(i32, Method, &str, Vec<EntityTag>, StatusCode)> = vec![
			(
				010,
				Method::GET,
				"/storage/user/a/b/c",
				vec![],
				StatusCode::OK,
			),
			(
				020,
				Method::DELETE,
				"/storage/user/a/b/c",
				vec![EntityTag::new(false, String::from("ANOTHER_ETAG"))],
				StatusCode::PRECONDITION_FAILED,
			),
			(
				030,
				Method::GET,
				"/storage/user/a/b/c",
				vec![],
				StatusCode::OK,
			),
			(
				040,
				Method::GET,
				"/storage/user/a/b/c",
				vec![],
				StatusCode::OK,
			),
			(
				050,
				Method::DELETE,
				"/storage/user/a/b/c",
				vec![EntityTag::new(false, String::from("A"))],
				StatusCode::OK,
			),
			(
				060,
				Method::GET,
				"/storage/user/a/b/c",
				vec![],
				StatusCode::NOT_FOUND,
			),
			(
				070,
				Method::DELETE,
				"/storage/user/a/b/c",
				vec![EntityTag::new(false, String::from("A"))],
				StatusCode::NOT_FOUND,
			),
			(
				080,
				Method::DELETE,
				"/storage/user/a/b/c",
				vec![EntityTag::new(false, String::from("ANOTHER_ETAG"))],
				StatusCode::NOT_FOUND,
			),
		];

		for test in tests {
			print!(
				"#{:03} : {} request to {} with If-Match = {:?} ... ",
				test.0, test.1, test.2, test.3
			);

			let request = actix_web::test::TestRequest::with_uri(test.2)
				.method(test.1.clone())
				.set(actix_web::http::header::IfMatch::Items(test.3.clone()))
				.to_request();
			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), test.4);

			println!("OK");
		}
	}
}
