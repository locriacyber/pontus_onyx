#[derive(Debug, PartialEq, Eq)]
pub enum PutError {
	GetError(super::super::GetError),
	DoesNotWorksForFolders,
	ContentNotChanged,
	CanNotSerializeFile {
		item_path: crate::item::ItemPath,
		error: String,
	},
	CanNotDeserializeFile {
		item_path: crate::item::ItemPath,
		error: String,
	},
	NoContentInside {
		item_path: crate::item::ItemPath,
	},
	InternalError,
}
impl std::fmt::Display for PutError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::GetError(get_error) => std::fmt::Display::fmt(get_error, f),
			Self::DoesNotWorksForFolders => f.write_str("this method does not works on folders"),
			Self::ContentNotChanged => f.write_str("content not changed"),
			Self::CanNotSerializeFile { item_path, error } => f.write_fmt(format_args!(
				"can not serialize file `{}` because : {}",
				item_path, error
			)),
			Self::CanNotDeserializeFile { item_path, error } => f.write_fmt(format_args!(
				"can not deserialize file `{}` because : {}",
				item_path, error
			)),
			Self::NoContentInside { item_path } => {
				f.write_fmt(format_args!("no content found in `{}`", item_path))
			}
			Self::InternalError => f.write_str("internal server error"),
		}
	}
}
impl std::error::Error for PutError {}
#[cfg(feature = "server")]
impl crate::database::Error for PutError {
	fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse {
		match self {
			// TODO : we have to find a way to change method
			Self::GetError(get_error) => {
				crate::database::Error::to_response(get_error, origin, should_have_body)
			}
			Self::DoesNotWorksForFolders => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::BAD_REQUEST,
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
			Self::CanNotSerializeFile { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				should_have_body,
			),
			Self::CanNotDeserializeFile { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				should_have_body,
			),
			Self::NoContentInside { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				should_have_body,
			),
			Self::InternalError => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				should_have_body,
			),
		}
	}
}
