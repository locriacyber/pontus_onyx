#[derive(Debug)]
pub enum ErrorGet {
	CanNotBeListed,
	Conflict,
	IfMatchNotFound,
	IfNoneMatch,
	NotFound,
	WrongPath,
	InternalError,
}
impl std::fmt::Display for ErrorGet {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		match self {
			Self::CanNotBeListed => f.write_str("the content of this folder can not be listed"),
			Self::Conflict => f.write_str(
				"there is a conflict of name between folder and document name on the request path",
			),
			Self::IfMatchNotFound => f.write_str(
				"the requested ETag was not found (specified in If-Match header of your request)",
			),
			Self::IfNoneMatch => f.write_str(
				"the unwanted ETag was found (specified in If-None-Match header of your request)",
			),
			Self::NotFound => f.write_str("requested item was not found"),
			Self::WrongPath => f.write_str("the path of the item is incorrect"),
			Self::InternalError => f.write_str("an internal error occured"),
		}
	}
}
impl std::error::Error for ErrorGet {}

#[cfg(feature = "server_bin")]
impl ErrorGet {
	pub fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse {
		let request_method = actix_web::http::Method::GET;
		match self {
			ErrorGet::CanNotBeListed => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			ErrorGet::Conflict => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::CONFLICT,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			ErrorGet::IfMatchNotFound => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			ErrorGet::IfNoneMatch => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			ErrorGet::NotFound => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			ErrorGet::WrongPath => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			ErrorGet::InternalError => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
		}
	}
}
