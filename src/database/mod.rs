mod delete;
mod get;
mod load;
mod put;
mod save;
mod utils;

pub use delete::ErrorDelete;
pub use get::ErrorGet;
pub use load::ErrorNewDatabase;
pub use put::{ErrorPut, ResultPut};
pub use save::{DataDocument, DataFolder, DataMonolyth};

#[cfg(feature = "server_bin")]
pub use save::do_not_handle_events;

#[derive(Debug)]
pub struct Database {
	content: crate::Item,
	changes_tx: std::sync::mpsc::Sender<crate::database::Event>,
}

#[derive(Debug, Clone)]
pub enum DataSource {
	Memory(crate::Item),
	File(std::path::PathBuf),
}

#[derive(Debug)]
pub enum Event {
	Create { path: String, item: crate::Item },
	Update { path: String, item: crate::Item },
	Delete { path: String },
}

impl Database {
	fn fetch_item(&self, request: &[&str]) -> Result<Option<&crate::Item>, FetchError> {
		let mut result = Some(&self.content);

		for &request_name in request.iter().filter(|&&e| !e.is_empty()) {
			if let Some(item) = result {
				match item {
					crate::Item::Folder {
						etag: _,
						content: folder_content,
					} => {
						result = folder_content.get(request_name).map(|b| &**b);
					}
					crate::Item::Document {
						etag: _,
						content: _,
						content_type: _,
						last_modified: _,
					} => {
						return Err(FetchError::FolderDocumentConflict);
					}
				}
			}
		}

		return Ok(result);
	}
	fn fetch_item_mut(&mut self, request: &[&str]) -> Result<Option<&mut crate::Item>, FetchError> {
		let mut result = Some(&mut self.content);

		for &request_name in request.iter().filter(|&&e| !e.is_empty()) {
			if let Some(item) = result {
				match item {
					crate::Item::Folder {
						etag: _,
						content: folder_content,
					} => {
						result = folder_content.get_mut(request_name).map(|b| &mut **b);
					}
					crate::Item::Document {
						etag: _,
						content: _,
						content_type: _,
						last_modified: _,
					} => {
						return Err(FetchError::FolderDocumentConflict);
					}
				}
			}
		}

		return Ok(result);
	}
	fn cleanup_empty_folders(&mut self, path: &str) -> Result<(), CleanupFolderError> {
		let splitted_path: Vec<&str> = path.split('/').collect();

		match self.fetch_item_mut(&splitted_path) {
			Ok(Some(crate::Item::Folder { etag: _, content })) => {
				if content.is_empty() && splitted_path.len() > 1 {
					let temp = self.fetch_item_mut(
						&splitted_path
							.iter()
							.take(splitted_path.len() - 1 - 1)
							.cloned()
							.collect::<Vec<&str>>(),
					);

					if let Ok(Some(crate::Item::Folder {
						etag: _,
						content: parent_content,
					})) = temp
					{
						parent_content.remove(
							*splitted_path
								.iter()
								.filter(|p| !p.is_empty())
								.last()
								.unwrap(),
						);
					}
				}

				Ok(())
			}
			_ => Err(CleanupFolderError::NotAFolder),
		}
	}
}

#[derive(Debug)]
enum FetchError {
	FolderDocumentConflict,
}

enum CleanupFolderError {
	NotAFolder,
}

#[cfg(feature = "server_bin")]
#[derive(serde::Serialize)]
struct JsonResponse {
	http_code: u16,
	#[serde(skip_serializing_if = "Option::is_none")]
	http_description: Option<&'static str>,
	#[serde(rename = "ETag", skip_serializing_if = "Option::is_none")]
	etag: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	hint: Option<String>,
}

#[cfg(feature = "server_bin")]
pub fn build_http_json_response(
	request_method: &actix_web::http::Method,
	code: actix_web::http::StatusCode,
	etag: Option<String>,
	hint: Option<String>,
	should_have_body: bool,
) -> actix_web::HttpResponse {
	let mut response = actix_web::HttpResponse::build(code);
	response.content_type("application/ld+json");
	if request_method == actix_web::http::Method::GET && code.is_success() {
		response.header("Cache-Control", "no-cache");
	}
	response.header("Access-Control-Allow-Origin", "*");

	let mut expose_headers = String::from("Content-Length, Content-Type");
	if etag.is_some() {
		expose_headers += ", ETag";
	}
	response.header("Access-Control-Expose-Headers", expose_headers);

	if let Some(etag) = &etag {
		response.header("ETag", etag.clone());
	}

	return if should_have_body {
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
