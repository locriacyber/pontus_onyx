#![allow(clippy::needless_return)]

extern crate pontus_onyx;

/*
TODO : continue to :
	https://datatracker.ietf.org/doc/html/draft-dejong-remotestorage-16
		"10. Application-first bearer token issuance"
*/

#[cfg(feature = "server")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
	println!("starting to listen to http://localhost:7541/");

	actix_web::HttpServer::new(|| {
		actix_web::App::new()
			.data(std::sync::Mutex::new(
				pontus_onyx::database::Database::from_bytes(&[]).unwrap(),
			))
			.service(get_item)
			.service(head_item)
			.service(options_item)
			.service(put_item)
			.service(delete_item)
	})
	.bind("localhost:7541")? // TODO : HTTPS
	.run()
	.await
}

/*
TODO :
	GET requests MAY have a comma-separated list of revisions in an
	'If-None-Match' header [COND], and SHOULD be responded to with a 304
	response if that list includes the document or folder's current
	version.
*/
#[cfg(feature = "server")]
fn get(paths: actix_web::web::Path<String>,
	database: actix_web::web::Data<std::sync::Mutex<pontus_onyx::database::Database>>,
	should_have_body: bool,
) -> actix_web::web::HttpResponse {
	let paths: Vec<&str> = paths.split("/").collect();

	let should_be_folder = paths.last().unwrap() == &"";

	return match database.lock().unwrap().read(&paths) {
		Ok(Some(item)) => {
			match item {
				pontus_onyx::Item::Folder { content } => {
					if should_be_folder {
						let mut items_result = serde_json::json!({});
						for (child_name, child) in content.iter().filter(|(_, e)| match &***e {
							pontus_onyx::Item::Document { content: _ } => true,
							pontus_onyx::Item::Folder { content } => {
								!content.is_empty() // TODO : recursive if child is also empty ?
							}
						}) {
							match &**child {
								pontus_onyx::Item::Folder { content: _ } => {
									items_result[format!("{}/", child_name)] = serde_json::json!({
										"ETag": "TODO",
									});
								}
								pontus_onyx::Item::Document {
									content: document_content,
								} => {
									items_result[child_name] = serde_json::json!({
										"ETag": "TODO",
										"Content-Type": "TODO",
										"Content-Length": document_content.len(),
										"Last-Modified": "TODO",
									});
								}
							}
						}

						actix_web::HttpResponse::Ok()
							.content_type("application/ld+json")
							.header("cache-control", "no-cache")
							.body(if should_have_body {
								format!(
									"{}",
									serde_json::json!({
										"@context": "http://remotestorage.io/spec/folder-description",
										"items": items_result,
									})
									.to_string()
								)
							} else {
								String::new()
							})
					} else {
						// TODO : help user to say there is a folder with this name ?
						actix_web::HttpResponse::NotFound()
							.content_type("application/ld+json")
							.body(if should_have_body {
								r#"{"http_code":404,"http_description":"requested content not found"}"#
							} else {
								""
							})
					}
				}
				pontus_onyx::Item::Document { content } => {
					if !should_be_folder {
						actix_web::HttpResponse::Ok()
							.header("ETag", "TODO")
							.header("cache-control", "no-cache")
							.content_type("text/plain") // TODO
							.body(if should_have_body {
								content
							} else {
								vec![]
							})
					} else {
						actix_web::HttpResponse::NotFound()
							.content_type("application/ld+json")
							.body(if should_have_body {
								r#"{"http_code":404,"http_description":"requested content not found"}"#
							} else {
								""
							})
					}
				}
			}
		}
		Ok(None) => actix_web::HttpResponse::NotFound()
			.content_type("application/ld+json")
			.body(if should_have_body {
				r#"{"http_code":404,"http_description":"requested content not found"}"#
			} else {
				""
			}),
		Err(pontus_onyx::database::ReadError::WrongPath) => actix_web::HttpResponse::BadRequest()
			.content_type("application/ld+json")
			.body(if should_have_body {
				r#"{"http_code":400,"http_description":"bad request"}"#
			} else {
				""
			}),
		Err(err) => {
			println!("ERROR : {:?} : {:?}", paths, err); // TODO
			actix_web::HttpResponse::InternalServerError()
				.content_type("application/ld+json")
				.body(if should_have_body {
					r#"{"http_code":500,"http_description":"internal server error"}"#
				} else {
					""
				})
		}
	};
}

#[cfg(feature = "server")]
#[actix_web::get("/{requested_item:.*}")]
async fn get_item(
	paths: actix_web::web::Path<String>,
	database: actix_web::web::Data<std::sync::Mutex<pontus_onyx::database::Database>>,
) -> actix_web::web::HttpResponse {
	return get(paths, database, true);
}

#[cfg(feature = "server")]
#[actix_web::head("/{requested_item:.*}")]
async fn head_item(
	paths: actix_web::web::Path<String>,
	database: actix_web::web::Data<std::sync::Mutex<pontus_onyx::database::Database>>,
) -> actix_web::web::HttpResponse {
	return get(paths, database, false);
}

/*
TODO :
	A successful OPTIONS request SHOULD be responded to as described in
	the CORS section below.
*/
/*
TODO :
	The server MUST also
	reply to preflight OPTIONS requests as per CORS.
*/
#[cfg(feature = "server")]
#[actix_web::options("/{requested_item:.*}")]
async fn options_item(
	_paths: actix_web::web::Path<String>,
	_database: actix_web::web::Data<std::sync::Mutex<pontus_onyx::database::Database>>,
) -> actix_web::web::HttpResponse {
	todo!()
}

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
	PUT and DELETE requests
	MAY have an 'If-Match' request header [COND], and MUST fail with a
	412 response code if that does not match the document's current
	version.
*/
/*
TODO :
	A PUT request MAY have an 'If-None-Match: *' header [COND],
	in which case it MUST fail with a 412 response code if the document
	already exists.
*/
/*
TODO :
	Unless [KERBEROS] is used (see section 10 below), all other
	requests SHOULD present a bearer token with sufficient access scope,
	using a header of the following form (no double quotes here):
		Authorization: Bearer <access_token>
*/
#[cfg(feature = "server")]
#[actix_web::put("/{requested_item:.*}")]
async fn put_item(
	mut request_payload: actix_web::web::Payload,
	request: actix_web::web::HttpRequest,
	paths: actix_web::web::Path<String>,
	database: actix_web::web::Data<std::sync::Mutex<pontus_onyx::database::Database>>,
) -> Result<actix_web::web::HttpResponse, actix_web::Error> {
	let paths: Vec<&str> = paths.split("/").collect();

	let mut body = actix_web::web::BytesMut::new();
	while let Some(content) = futures::StreamExt::next(&mut request_payload).await {
		let content = content?;
		body.extend_from_slice(&content);
	}
	let body = body.freeze();

	let content_type = request.headers().get("content-type");

	if let None = content_type {
		return Ok(actix_web::HttpResponse::BadRequest()
			.content_type("application/ld+json")
			.body(
				r#"{"http_code":400,"http_description":"bad request","hint":"missing content-type HTTP header"}"#,
			));
	}

	/*
	TODO :
		* its version being updated, as well as that of its parent folder
			and further ancestor folders, using a strong validator [HTTP,
			section 7.2].
	*/
	let ETag = "TODO";

	return Ok(
		match database.lock().unwrap().update(
			&paths,
			pontus_onyx::Item::Document {
				content: body.to_vec(),
			},
		) {
			Ok(new_ETag) => actix_web::HttpResponse::Ok()
				.content_type("application/ld+json")
				.body(
					format!(r#"{{"http_code":200,"http_description":"success","ETag":"{}"}}"#, new_ETag),
				),
			Err(pontus_onyx::database::UpdateError::WrongPath) => {
				actix_web::HttpResponse::BadRequest()
					.content_type("application/ld+json")
					.body(r#"{"http_code":400,"http_description":"bad request"}"#)
			}
			Err(pontus_onyx::database::UpdateError::FolderDocumentConflict) => {
				actix_web::HttpResponse::Conflict()
					.content_type("application/ld+json")
					.body(r#"{"http_code":409,"http_description":"conflict"}"#)
			}
			Err(pontus_onyx::database::UpdateError::NotFound) => {
				actix_web::HttpResponse::NotFound()
					.content_type("application/ld+json")
					.body(r#"{"http_code":404,"http_description":"requested content not found"}"#)
			}
			Err(_TODO) => actix_web::HttpResponse::InternalServerError()
				.content_type("application/ld+json")
				.body(r#"{"http_code":500,"http_description":"internal server error"}"#),
		},
	);
}

#[cfg(feature = "server")]
#[actix_web::delete("/{requested_item:.*}")]
async fn delete_item(
	paths: actix_web::web::Path<String>,
	database: actix_web::web::Data<std::sync::Mutex<pontus_onyx::database::Database>>,
) -> actix_web::web::HttpResponse {
	let paths: Vec<&str> = paths.split("/").collect();

	return match database.lock().unwrap().delete(&paths) {
		Ok(ETag) => {
			actix_web::HttpResponse::Ok()
				.content_type("application/ld+json")
				.header("ETag", ETag)
				.finish()
				/*
				TODO ?
				.body(
					format!(r#"{{"http_code":200,"http_description":"success","ETag":"{}"}}"#, new_ETag),
				)
				*/
		}
		Err(pontus_onyx::database::DeleteError::WrongPath) => {
			actix_web::HttpResponse::BadRequest()
				.content_type("application/ld+json")
				.body(r#"{"http_code":400,"http_description":"bad request"}"#)
		}
		Err(pontus_onyx::database::DeleteError::FolderDocumentConflict) => {
			actix_web::HttpResponse::Conflict()
				.content_type("application/ld+json")
				.body(r#"{"http_code":409,"http_description":"conflict"}"#)
		}
		Err(pontus_onyx::database::DeleteError::NotFound) => {
			actix_web::HttpResponse::NotFound()
				.content_type("application/ld+json")
				.body(r#"{"http_code":404,"http_description":"requested content not found"}"#)
		}
		Err(_TODO) => actix_web::HttpResponse::InternalServerError()
			.content_type("application/ld+json")
			.body(r#"{"http_code":500,"http_description":"internal server error"}"#),
	};
}

/*
TODO ?
	Servers MAY support Content-Range headers [RANGE] on GET requests,
	but whether or not they do SHOULD be announced both through the
	"http://tools.ietf.org/html/rfc7233" option mentioned below in
	section 10 and through the HTTP 'Accept-Ranges' response header.
*/

/*
TODO :
* 304 for a conditional GET request whose precondition
		fails (see "Versioning" below),
* 401 for all requests that require a valid bearer token and
		where no valid one was sent (see also [BEARER, section
		3.1]),
* 403 for all requests that have insufficient scope, e.g.
		accessing a <module> for which no scope was obtained, or
		accessing data outside the user's <storage_root>,
* 412 for a conditional PUT or DELETE request whose precondition
		fails (see "Versioning" below),
* 413 if the payload is too large, e.g. when the server has a
		maximum upload size for documents
* 414 if the request URI is too long,
* 416 if Range requests are supported by the server and the Range
		request can not be satisfied,
* 429 if the client makes too frequent requests or is suspected
		of malicious activity,
* 4xx for all malformed requests, e.g. reserved characters in the
		path [URI, section 2.2], as well as for all PUT and DELETE
		requests to folders,
* 507 in case the account is over its storage quota,
*/
/*
TODO :
	All responses MUST carry CORS headers [CORS].
*/
