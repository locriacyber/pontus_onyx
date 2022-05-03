use sources::DataSource;

pub mod sources;

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
		path: &crate::item::ItemPath,
		if_match: &crate::item::Etag,
		if_none_match: &[&crate::item::Etag],
	) -> Result<crate::item::Item, Box<dyn std::error::Error>> {
		self.source.get(path, if_match, if_none_match, true)
	}

	pub fn put(
		&mut self,
		path: &crate::item::ItemPath,
		content: crate::item::Item,
		if_match: &crate::item::Etag,
		if_none_match: &[&crate::item::Etag],
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
		path: &crate::item::ItemPath,
		if_match: &crate::item::Etag,
	) -> Result<crate::item::Etag, Box<dyn std::error::Error>> {
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
	Created(crate::item::Etag),
	Updated(crate::item::Etag),
	Err(Box<dyn std::error::Error>),
}
impl PutResult {
	pub fn unwrap(self) -> crate::item::Etag {
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
	etag: Option<crate::item::Etag>,
	#[serde(skip_serializing_if = "Option::is_none")]
	hint: Option<String>,
}

#[cfg(feature = "server_bin")]
pub fn build_http_json_response(
	origin: &str,
	request_method: &actix_web::http::Method,
	code: actix_web::http::StatusCode,
	etag: Option<crate::item::Etag>,
	hint: Option<String>,
	should_have_body: bool,
) -> actix_web::HttpResponse {
	let mut response = actix_web::HttpResponse::build(code);
	response.content_type("application/ld+json");
	if request_method == actix_web::http::Method::GET && code.is_success() {
		response.insert_header((actix_web::http::header::CACHE_CONTROL, "no-cache"));
	}
	response.insert_header((actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin));

	if origin != "*" {
		response.insert_header((actix_web::http::header::VARY, "Origin"));
	}

	let mut expose_headers = String::from("Content-Length, Content-Type");
	if etag.is_some() {
		expose_headers += ", ETag";
	}
	response.insert_header((
		actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
		expose_headers,
	));

	if let Some(etag) = &etag {
		let etag: String = (*etag).clone().into();
		response.insert_header((actix_web::http::header::ETAG, etag));
	}

	// TODO : response.insert_header((actix_web::http::header::LAST_MODIFIED, last_modified));

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
