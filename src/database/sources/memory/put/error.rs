#[derive(Debug, PartialEq, Eq)]
pub enum PutError {
	GetError(super::super::GetError),
	NoContentInside {
		item_path: crate::item::ItemPath,
	},
	DoesNotWorksForFolders,
	InternalError,
	ContentNotChanged,
	CanNotFetchParent {
		item_path: crate::item::ItemPath,
		error: super::super::GetError,
	},
}
impl std::fmt::Display for PutError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::GetError(error) => f.write_fmt(format_args!("{}", error)),
			Self::NoContentInside { item_path } => {
				f.write_fmt(format_args!("no content found in `{}`", item_path))
			}
			Self::DoesNotWorksForFolders => f.write_str("this method does not works on folders"),
			Self::InternalError => f.write_str("internal server error"),
			Self::ContentNotChanged => f.write_str("content not changed"),
			Self::CanNotFetchParent { item_path, error } => f.write_fmt(format_args!(
				"can not fetch parent of `{}`, because : `{}`",
				item_path, error
			)),
		}
	}
}
impl std::error::Error for PutError {}
#[cfg(feature = "server_bin")]
impl crate::database::Error for PutError {
	fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse {
		match self {
			Self::GetError(error) => error.to_response(origin, should_have_body),
			Self::NoContentInside { item_path: _ } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::DoesNotWorksForFolders => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::InternalError => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::ContentNotChanged => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::NOT_MODIFIED,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::CanNotFetchParent {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
		}
	}
}
