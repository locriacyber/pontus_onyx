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

	let found = database.lock().unwrap().read(&path)?;
	if let Some(none_match) = request.headers().get("If-None-Match") {
		let mut none_match = none_match
			.to_str()
			.unwrap()
			.split(',')
			.map(|s| s.trim().replace('"', ""));

		if let Some(pontus_onyx::Item::Document {
			etag: document_etag,
			content: _,
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

		match database.lock().unwrap().update(
			&path,
			pontus_onyx::Item::Document {
				etag: ulid::Ulid::new().to_string(),
				content: body.to_vec(),
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
		match database.lock().unwrap().create(&path, &body) {
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
