pub mod sources;

use sources::DataSource;

#[cfg(feature = "server_file_storage")]
pub use sources::FolderStorage;
#[cfg(feature = "server_local_storage")]
pub use sources::LocalStorage;
pub use sources::MemoryStorage;

#[derive(Debug)]
pub struct Database {
	source: Box<dyn DataSource>,
}
impl Database {
	pub fn new(source: Box<dyn DataSource>) -> Self {
		Database { source }
	}

	pub fn get(
		&self,
		path: &std::path::Path,
		if_match: &crate::Etag,
		if_none_match: &[&crate::Etag],
	) -> Result<crate::Item, Box<dyn std::error::Error>> {
		self.source.get(path, if_match, if_none_match, true)
	}

	pub fn put(
		&mut self,
		path: &std::path::Path,
		content: crate::Item,
		if_match: &crate::Etag,
		if_none_match: &[&crate::Etag],
	) -> PutResult {
		/*
		TODO :
			* its version being updated, as well as that of its parent folder
				and further ancestor folders, using a strong validator [HTTP,
				section 7.2].
		*/

		self.source.put(path, if_match, if_none_match, content)
	}

	pub fn delete(
		&mut self,
		path: &std::path::Path,
		if_match: &crate::Etag,
	) -> Result<crate::Etag, Box<dyn std::error::Error>> {
		/*
		TODO : option to keep old documents ?
			A provider MAY offer version rollback functionality to its users,
			but this specification does not define the interface for that.
		*/

		self.source.delete(path, if_match)
	}
}

#[derive(Debug)]
#[must_use = "this `PutResult` may be an `Err` variant, which should be handled"]
pub enum PutResult {
	Created(crate::Etag),
	Updated(crate::Etag),
	Err(Box<dyn std::error::Error>),
}
impl PutResult {
	pub fn unwrap(self) -> crate::Etag {
		match self {
			Self::Created(etag) => etag,
			Self::Updated(etag) => etag,
			Self::Err(e) => panic!("{}", e),
		}
	}
	pub fn unwrap_err(self) -> Box<dyn std::error::Error> {
		match self {
			Self::Created(etag) => panic!("found Created({})", etag),
			Self::Updated(etag) => panic!("found Updated({})", etag),
			Self::Err(e) => e,
		}
	}
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

#[cfg(feature = "server_bin")]
pub trait Error: std::fmt::Debug + std::error::Error {
	fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse;
}
