#![allow(clippy::needless_return)]

extern crate pontus_onyx;

/*
TODO : continue to :
	https://datatracker.ietf.org/doc/html/draft-dejong-remotestorage-16
		"A successful PUT request to a document MUST result in:"
*/

#[cfg(feature = "server")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
	actix_web::HttpServer::new(|| {
		actix_web::App::new()
			.data(pontus_onyx::database::Database::from_bytes(&[]).unwrap())
			.service(get_item)
			.service(put_item)
			.service(delete_item)
	})
	.bind("localhost:7541")?
	.run()
	.await
}

#[cfg(feature = "server")]
#[actix_web::get("/{requested_item:.*}")]
async fn get_item(
	paths: actix_web::web::Path<String>,
	database: actix_web::web::Data<pontus_onyx::database::Database>,
) -> impl actix_web::Responder {
	let paths: Vec<&str> = paths.split("/").collect();

	let should_be_folder = paths.last().unwrap() == &"";

	return match database.get(&paths) {
		Ok(Some(item)) => {
			match item {
				pontus_onyx::Item::Folder { name: _, content } => {
					if should_be_folder {
						let mut items_result = serde_json::json!({});
						for child in content.values().filter(|&e| match &**e {
							pontus_onyx::Item::Document {
								name: _,
								content: _,
							} => true,
							pontus_onyx::Item::Folder { name: _, content } => {
								!content.is_empty() // TODO : recursive if child is also empty ?
							}
						}) {
							match &**child {
								pontus_onyx::Item::Folder {
									name: folder_name,
									content: _,
								} => {
									items_result[format!("{}/", folder_name)] = serde_json::json!({
										"ETag": "TODO",
									});
								}
								pontus_onyx::Item::Document {
									name: document_name,
									content: document_content,
								} => {
									items_result[document_name] = serde_json::json!({
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
							.body(format!(
								"{}",
								serde_json::json!({
									"@context": "http://remotestorage.io/spec/folder-description",
									"items": items_result,
								})
								.to_string()
							))
					} else {
						// TODO : help user to say there is a folder with this name ?
						actix_web::HttpResponse::NotFound()
							.content_type("application/ld+json")
							.body(
								r#"{"error_code":404,"error_description":"requested content not found"}"#,
							)
					}
				}
				pontus_onyx::Item::Document { name: _, content } => actix_web::HttpResponse::Ok()
					.header("ETag", "TODO")
					.content_type("text/plain") // TODO
					.body(content),
			}
		}
		Ok(None) => actix_web::HttpResponse::NotFound()
			.content_type("application/ld+json")
			.body(r#"{"error_code":404,"error_description":"requested content not found"}"#),
		Err(pontus_onyx::database::GetError::WrongPath) => actix_web::HttpResponse::BadRequest()
			.content_type("application/ld+json")
			.body(r#"{"error_code":400,"error_description":"bad request"}"#),
		Err(err) => {
			println!("ERROR : {:?} : {:?}", paths, err); // TODO
			actix_web::HttpResponse::InternalServerError()
				.content_type("application/ld+json")
				.body(r#"{"error_code":500,"error_description":"internal server error"}"#)
		}
	};
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
#[cfg(feature = "server")]
#[actix_web::put("/{requested_item:.*}")]
async fn put_item(
	_paths: actix_web::web::Path<String>,
	_database: actix_web::web::Data<pontus_onyx::database::Database>,
) -> impl actix_web::Responder {
	// TODO
	actix_web::HttpResponse::InternalServerError()
		.content_type("application/ld+json")
		.body(r#"{"error_code":500,"error_description":"internal server error"}"#)
}
#[cfg(feature = "server")]
#[actix_web::delete("/{requested_item:.*}")]
async fn delete_item(
	_paths: actix_web::web::Path<String>,
	_database: actix_web::web::Data<pontus_onyx::database::Database>,
) -> impl actix_web::Responder {
	// TODO
	actix_web::HttpResponse::InternalServerError()
		.content_type("application/ld+json")
		.body(r#"{"error_code":500,"error_description":"internal server error"}"#)
}

/*
TODO ?
	Servers MAY support Content-Range headers [RANGE] on GET requests,
	but whether or not they do SHOULD be announced both through the
	"http://tools.ietf.org/html/rfc7233" option mentioned below in
	section 10 and through the HTTP 'Accept-Ranges' response header.
*/
