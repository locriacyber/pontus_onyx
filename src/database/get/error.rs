#[derive(Debug)]
pub enum ErrorGet {
	CanNotBeListed,
	Conflict,
	IfMatchNotFound,
	IfNoneMatch,
	NotFound,
	WrongPath,
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
		}
	}
}
impl std::error::Error for ErrorGet {}

#[cfg(feature = "server_bin")]
impl std::convert::From<ErrorGet> for actix_web::HttpResponse {
	fn from(input: ErrorGet) -> Self {
		let request_method = actix_web::http::Method::GET;
		match input {
			ErrorGet::CanNotBeListed => crate::database::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorGet::Conflict => crate::database::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::CONFLICT,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorGet::IfMatchNotFound => crate::database::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorGet::IfNoneMatch => crate::database::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorGet::NotFound => crate::database::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorGet::WrongPath => crate::database::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", input)),
				true,
			),
		}
	}
}
