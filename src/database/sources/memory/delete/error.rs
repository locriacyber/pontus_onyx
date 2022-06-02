#[derive(Debug, PartialEq)]
pub enum DeleteError {
	Conflict {
		item_path: crate::item::ItemPath,
	},
	DoesNotWorksForFolders,
	NotFound {
		item_path: crate::item::ItemPath,
	},
	NoContentInside {
		item_path: crate::item::ItemPath,
	},
	IncorrectItemName {
		item_path: crate::item::ItemPath,
		error: String,
	},
	NoIfMatch {
		item_path: crate::item::ItemPath,
		search: crate::item::Etag,
		found: crate::item::Etag,
	},
}
impl std::fmt::Display for DeleteError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Conflict { item_path } => f.write_fmt(format_args!("name conflict between folder and file on the path `{}`", item_path)),
			Self::DoesNotWorksForFolders => f.write_str("this method does not works on folders"),
			Self::NotFound { item_path } => f.write_fmt(format_args!("path not found : `{}`", item_path)),
			Self::NoContentInside { item_path } => f.write_fmt(format_args!("no content found in `{}`", item_path)),
			Self::IncorrectItemName { item_path, error } => f.write_fmt(format_args!("the path `{}` is incorrect, because {}", item_path, error)),
			Self::NoIfMatch { item_path, search, found } => f.write_fmt(format_args!("the requested `{}` etag (through `IfMatch`) for `{}` was not found, found `{}` instead", search, item_path, found)),
		}
	}
}
impl std::error::Error for DeleteError {}
#[cfg(feature = "server_bin")]
impl crate::database::Error for DeleteError {
	fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse {
		match self {
			Self::Conflict { item_path: _ } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::CONFLICT,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::DoesNotWorksForFolders => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::NotFound { item_path: _ } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::NoContentInside { item_path: _ } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::IncorrectItemName {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::NoIfMatch {
				item_path: _,
				search: _,
				found: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
		}
	}
}
