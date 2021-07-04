mod delete;
mod get;
mod put;
mod save;
mod sources;
mod utils;

pub use put::ResultPut;
pub use save::{DataDocument, DataFolder};
pub use sources::*;

#[derive(Debug)]
pub struct Database {
	pub source: DataSource,
}

#[derive(Debug)]
enum FetchError {
	// FolderDocumentConflict,
}

#[cfg(feature = "server_bin")]
#[derive(serde::Serialize)]
struct JsonResponse {
	http_code: u16,
	#[serde(skip_serializing_if = "Option::is_none")]
	http_description: Option<&'static str>,
	#[serde(rename = "ETag", skip_serializing_if = "Option::is_none")]
	etag: Option<crate::Etag>,
	#[serde(skip_serializing_if = "Option::is_none")]
	hint: Option<String>,
}

#[cfg(feature = "server_bin")]
pub fn build_http_json_response(
	origin: &str,
	request_method: &actix_web::http::Method,
	code: actix_web::http::StatusCode,
	etag: Option<crate::Etag>,
	hint: Option<String>,
	should_have_body: bool,
) -> actix_web::HttpResponse {
	let mut response = actix_web::HttpResponse::build(code);
	response.content_type("application/ld+json");
	if request_method == actix_web::http::Method::GET && code.is_success() {
		response.header(actix_web::http::header::CACHE_CONTROL, "no-cache");
	}
	response.header(actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);

	if origin != "*" {
		response.header(actix_web::http::header::VARY, "Origin");
	}

	let mut expose_headers = String::from("Content-Length, Content-Type");
	if etag.is_some() {
		expose_headers += ", ETag";
	}
	response.header(
		actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
		expose_headers,
	);

	if let Some(etag) = &etag {
		let etag: String = (*etag).clone().into();
		response.header(actix_web::http::header::ETAG, etag);
	}

	return if should_have_body || request_method != actix_web::http::Method::HEAD {
		response.body(
			serde_json::to_string(&JsonResponse {
				http_code: code.as_u16(),
				http_description: code.canonical_reason(),
				etag,
				hint,
			})
			.unwrap(),
		)
	} else {
		response.finish()
	};
}

pub trait Error: std::fmt::Debug + std::error::Error {
	fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse;
}
