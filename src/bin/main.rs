#![allow(clippy::needless_return)]

extern crate pontus_onyx;

/*
TODO :
	actix services tests
*/

/*
TODO : continue to :
	https://datatracker.ietf.org/doc/html/draft-dejong-remotestorage-16
		"12. Example wire transcripts"
*/

#[cfg(feature = "server")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
	println!("starting to listen to http://localhost:7541/");

	let database = std::sync::Arc::new(std::sync::Mutex::new(
		pontus_onyx::database::Database::from_bytes(&[]).unwrap(),
	));

	actix_web::HttpServer::new(move || {
		actix_web::App::new()
			.wrap(Auth {})
			.data(database.clone())
			.service(webfinger)
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
fn get(
	path: actix_web::web::Path<String>,
	database: actix_web::web::Data<
		std::sync::Arc<std::sync::Mutex<pontus_onyx::database::Database>>,
	>,
	should_have_body: bool,
) -> actix_web::web::HttpResponse {
	let should_be_folder = path.split('/').last().unwrap() == "";

	return match database.lock().unwrap().read(&path) {
		Ok(Some(item)) => {
			match item {
				pontus_onyx::Item::Folder { etag: _, content } => {
					if should_be_folder {
						let mut items_result = serde_json::json!({});
						for (child_name, child) in content.iter().filter(|(_, e)| match &***e {
							pontus_onyx::Item::Document {
								etag: _,
								content: _,
							} => true,
							pontus_onyx::Item::Folder { etag: _, content } => {
								!content.is_empty() // TODO : recursive if child is also empty ?
							}
						}) {
							match &**child {
								pontus_onyx::Item::Folder { etag, content: _ } => {
									items_result[format!("{}/", child_name)] = serde_json::json!({
										"ETag": etag,
									});
								}
								pontus_onyx::Item::Document {
									etag,
									content: document_content,
								} => {
									items_result[child_name] = serde_json::json!({
										"ETag": etag,
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
								serde_json::json!({
									"@context": "http://remotestorage.io/spec/folder-description",
									"items": items_result,
								})
								.to_string()
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
				pontus_onyx::Item::Document { etag, content } => {
					if !should_be_folder {
						actix_web::HttpResponse::Ok()
							.header("ETag", etag)
							.header("cache-control", "no-cache")
							.content_type("text/plain") // TODO
							.body(if should_have_body { content } else { vec![] })
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
			println!("ERROR : {:?} : {:?}", path, err); // TODO
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
struct Auth;

#[cfg(feature = "server")]
impl<S> actix_web::dev::Transform<S> for Auth
where
	S: actix_web::dev::Service<
		Request = actix_web::dev::ServiceRequest,
		Response = actix_web::dev::ServiceResponse<actix_web::dev::Body>,
		Error = actix_web::Error,
	>,
	S::Future: 'static,
{
	type Request = actix_web::dev::ServiceRequest;
	type Response = actix_web::dev::ServiceResponse<actix_web::dev::Body>;
	type Error = actix_web::Error;
	type InitError = ();
	type Transform = AuthMiddleware<S>;
	type Future = futures::future::Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		futures::future::ok(Self::Transform { service })
	}
}

#[cfg(feature = "server")]
struct AuthMiddleware<S> {
	service: S,
}

#[cfg(feature = "server")]
impl<S> actix_web::dev::Service for AuthMiddleware<S>
where
	S: actix_web::dev::Service<
		Request = actix_web::dev::ServiceRequest,
		Response = actix_web::dev::ServiceResponse<actix_web::dev::Body>,
		Error = actix_web::Error,
	>,
	S::Future: 'static,
{
	type Request = actix_web::dev::ServiceRequest;
	type Response = actix_web::dev::ServiceResponse<actix_web::dev::Body>;
	type Error = actix_web::Error;
	type Future =
		std::pin::Pin<Box<dyn futures::Future<Output = Result<Self::Response, Self::Error>>>>;

	fn poll_ready(
		&mut self,
		ctx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), Self::Error>> {
		self.service.poll_ready(ctx)
	}

	fn call(&mut self, service_request: Self::Request) -> Self::Future {
		match service_request.head().headers().get("Authorization") {
			Some(auth_value) => {
				// TODO : check value of the <Authorization> header
				if auth_value == "Bearer TODO" {
					let future = self.service.call(service_request);
					Box::pin(async move { Ok(future.await?) })
				} else {
					Box::pin(async move {
						Ok(actix_web::dev::ServiceResponse::new(
							service_request.into_parts().0,
							actix_web::HttpResponse::Unauthorized()
								.content_type("application/ld+json")
								.body(r#"{"http_code":401,"http_description":"unauthorized"}"#),
						))
					})
				}
			}
			None => {
				if service_request.path().starts_with("/public/")
					&& service_request
						.path()
						.split('/')
						.collect::<Vec<&str>>()
						.last()
						.unwrap_or(&"") != &""
					&& (service_request.method() == actix_web::http::Method::GET
						|| service_request.method() == actix_web::http::Method::HEAD
						|| service_request.method() == actix_web::http::Method::OPTIONS)
				{
					let future = self.service.call(service_request);
					Box::pin(async move { Ok(future.await?) })
				} else {
					Box::pin(async move {
						Ok(actix_web::dev::ServiceResponse::new(
							service_request.into_parts().0,
							actix_web::HttpResponse::Unauthorized()
								.content_type("application/ld+json")
								.body(r#"{"http_code":401,"http_description":"unauthorized"}"#),
						))
					})
				}
			}
		}
	}
}

#[cfg(feature = "server")]
#[actix_web::get("/{requested_item:.*}")]
async fn get_item(
	path: actix_web::web::Path<String>,
	database: actix_web::web::Data<
		std::sync::Arc<std::sync::Mutex<pontus_onyx::database::Database>>,
	>,
) -> actix_web::web::HttpResponse {
	return get(path, database, true);
}

#[cfg(feature = "server")]
#[actix_web::head("/{requested_item:.*}")]
async fn head_item(
	path: actix_web::web::Path<String>,
	database: actix_web::web::Data<
		std::sync::Arc<std::sync::Mutex<pontus_onyx::database::Database>>,
	>,
) -> actix_web::web::HttpResponse {
	return get(path, database, false);
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
	_path: actix_web::web::Path<String>,
	_database: actix_web::web::Data<
		std::sync::Arc<std::sync::Mutex<pontus_onyx::database::Database>>,
	>,
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
	path: actix_web::web::Path<String>,
	database: actix_web::web::Data<
		std::sync::Arc<std::sync::Mutex<pontus_onyx::database::Database>>,
	>,
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

	/*
	TODO :
		* its version being updated, as well as that of its parent folder
			and further ancestor folders, using a strong validator [HTTP,
			section 7.2].
	*/
	return Ok(
		match database.lock().unwrap().update(
			&path,
			pontus_onyx::Item::Document {
				etag: ulid::Ulid::new().to_string(),
				content: body.to_vec(),
			},
		) {
			Ok(new_etag) => actix_web::HttpResponse::Ok()
				.content_type("application/ld+json")
				.header("ETag", new_etag.clone())
				.body(format!(
					r#"{{"http_code":200,"http_description":"success","ETag":"{}"}}"#,
					new_etag
				)),
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
	database: actix_web::web::Data<
		std::sync::Arc<std::sync::Mutex<pontus_onyx::database::Database>>,
	>,
) -> actix_web::web::HttpResponse {
	return match database.lock().unwrap().delete(&paths) {
		Ok(etag) => {
			actix_web::HttpResponse::Ok()
				.content_type("application/ld+json")
				.header("ETag", etag)
				.finish()
			/*
			TODO ?
			.body(
				format!(r#"{{"http_code":200,"http_description":"success","ETag":"{}"}}"#, etag),
			)
			*/
		}
		Err(pontus_onyx::database::DeleteError::WrongPath) => actix_web::HttpResponse::BadRequest()
			.content_type("application/ld+json")
			.body(r#"{"http_code":400,"http_description":"bad request"}"#),
		Err(pontus_onyx::database::DeleteError::FolderDocumentConflict) => {
			actix_web::HttpResponse::Conflict()
				.content_type("application/ld+json")
				.body(r#"{"http_code":409,"http_description":"conflict"}"#)
		}
		Err(pontus_onyx::database::DeleteError::NotFound) => actix_web::HttpResponse::NotFound()
			.content_type("application/ld+json")
			.body(r#"{"http_code":404,"http_description":"requested content not found"}"#),
		Err(_TODO) => actix_web::HttpResponse::InternalServerError()
			.content_type("application/ld+json")
			.body(r#"{"http_code":500,"http_description":"internal server error"}"#),
	};
}

#[derive(serde::Deserialize)]
struct WebfingerQuery {}

#[cfg(feature = "server")]
#[actix_web::get("/.well-known/webfinger")]
async fn webfinger(_query: actix_web::web::Query<WebfingerQuery>) -> actix_web::web::HttpResponse {
	actix_web::HttpResponse::Ok()
		.content_type("application/ld+json")
		.body(
			format!(r#"{{"href":"/","rel":"http://tools.ietf.org/id/draft-dejong-remotestorage","properties":{{"http://remotestorage.io/spec/version":"{}","http://tools.ietf.org/html/rfc6749#section-4.2":{}}}}}"#,
				"draft-dejong-remotestorage-16",
				"TODO"
			)
		)
	/*
	TODO :
		If <auth-dialog> is a URL, the user can supply their credentials
		for accessing the account (how, is out of scope), and allow or
		reject a request by the connecting application to obtain a bearer
		token for a certain list of access scopes.
	*/
	/*
	TODO :
		Non-breaking examples that have been proposed so far, include a
		"http://tools.ietf.org/html/rfc6750#section-2.3" property, set to
		the string value "true" if the server supports passing the bearer
		token in the URI query parameter as per section 2.3 of [BEARER],
		instead of in the request header.
	*/
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
/*
TODO :
	A "http://remotestorage.io/spec/web-authoring" property has been
	proposed with a string value of the fully qualified domain name to
	which web authoring content is published if the server supports web
	authoring as per [AUTHORING]. Note that this extension is a breaking
	extension in the sense that it divides users into "haves", whose
	remoteStorage accounts allow them to author web content, and
	"have-nots", whose remoteStorage account does not support this
	functionality.
*/
/*
TODO :
	The server MAY expire bearer tokens, and MAY require the user to
	register applications as OAuth clients before first use; if no
	client registration is required, the server MUST ignore the value of
	the client_id parameter in favor of relying on the origin of the
	redirect_uri parameter for unique client identification. See section
	4 of [ORIGIN] for computing the origin.
*/
/*
TODO :
	11. Storage-first bearer token issuance

	To request that the application connects to the user account
	<account> ' ' <host>, providers MAY redirect to applications with a
	'remotestorage' field in the URL fragment, with the user account as
	value.

	The appplication MUST make sure this request is intended by the
	user. It SHOULD ask for confirmation from the user whether they want
	to connect to the given provider account. After confirmation, it
	SHOULD connect to the given provider account, as defined in Section
	10.

	If the 'remotestorage' field exists in the URL fragment, the
	application SHOULD ignore any other parameters such as
	'access_token' or 'state'
*/
