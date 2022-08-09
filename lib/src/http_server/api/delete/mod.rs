use std::sync::{Arc, Mutex};

#[actix_web::delete("/storage/{requested_item:.*}")]
pub async fn delete_item(
	path: actix_web::web::Path<String>,
	request: actix_web::HttpRequest,
	database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<crate::database::Database>>>,
	logger: actix_web::web::Data<Arc<Mutex<charlie_buffalo::Logger>>>,
) -> impl actix_web::Responder {
	// TODO : check security issue about this ?
	let all_origins = actix_web::http::header::HeaderValue::from_bytes(b"*").unwrap();
	let origin = request
		.headers()
		.get(actix_web::http::header::ORIGIN)
		.unwrap_or(&all_origins)
		.to_str()
		.unwrap();

	let local_path = crate::item::ItemPath::from(path.into_inner().as_str());

	match database.lock().unwrap().delete(
		&local_path,
		super::convert_actix_if_match(&request)
			.first()
			.unwrap_or(&crate::item::Etag::from("")),
	) {
		Ok(etag) => {
			return crate::database::build_http_json_response(
				origin,
				request.method(),
				actix_web::http::StatusCode::OK,
				Some(etag),
				None,
				None,
				true,
			);
		}
		Err(e) => {
			if e.is::<crate::database::sources::memory::DeleteError>() {
				return crate::database::Error::to_response(
					&*e.downcast::<crate::database::sources::memory::DeleteError>()
						.unwrap(),
					origin,
					true,
				);
			} else if e.is::<crate::database::sources::folder::DeleteError>() {
				return crate::database::Error::to_response(
					&*e.downcast::<crate::database::sources::folder::DeleteError>()
						.unwrap(),
					origin,
					true,
				);
			} else {
				logger.lock().unwrap().push(
					vec![
						(String::from("level"), String::from("ERROR")),
						(String::from("module"), String::from("https?")),
						(String::from("method"), String::from("DELETE")),
						(String::from("path"), local_path.to_string()),
					],
					Some(&format!("error from database : {e}")),
				);

				return crate::database::build_http_json_response(
					origin,
					request.method(),
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
					None,
					None,
					true,
				);
			}
		}
	}
}

#[cfg(test)]
mod tests;
