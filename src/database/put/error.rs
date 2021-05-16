#[derive(Debug)]
pub enum ErrorPut {
	Conflict,
	IfMatchNotFound,
	IfNoneMatch,
	InternalError,
	NotFound,
	NotModified,
	WorksOnlyForDocument,
	WrongPath,
}
impl std::fmt::Display for ErrorPut {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		match self {
			Self::Conflict => f.write_str(
				"there is a conflict of name between folder and document name on the request path",
			),
			Self::IfMatchNotFound => f.write_str(
				"the requested ETag was not found (specified in If-Match header of your request)",
			),
			Self::IfNoneMatch => f.write_str(
				"the unwanted ETag was found (specified in If-None-Match header of your request)",
			),
			Self::InternalError => {
				f.write_str("there is an internal error that should not logically happen")
			}
			Self::NotFound => f.write_str("requested item was not found"),
			Self::NotModified => f.write_str("this document was not modified"),
			Self::WorksOnlyForDocument => f.write_str("this method works only on documents"),
			Self::WrongPath => f.write_str("the path of the item is incorrect"),
		}
	}
}
impl std::error::Error for ErrorPut {}

#[cfg(feature = "server_bin")]
impl std::convert::From<ErrorPut> for actix_web::HttpResponse {
	fn from(input: ErrorPut) -> Self {
		let request_method = actix_web::http::Method::PUT;
		match input {
			ErrorPut::Conflict => crate::database::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::CONFLICT,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorPut::IfMatchNotFound => crate::database::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorPut::IfNoneMatch => crate::database::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorPut::InternalError => crate::database::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorPut::NotFound => crate::database::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorPut::NotModified => crate::database::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::NOT_MODIFIED,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorPut::WorksOnlyForDocument => crate::database::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorPut::WrongPath => crate::database::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", input)),
				true,
			),
		}
	}
}
