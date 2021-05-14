#[derive(Debug)]
pub enum ErrorDelete {
	Conflict,
	IfMatchNotFound,
	IfNoneMatch,
	InternalError,
	NotFound,
	WorksOnlyForDocument,
	WrongPath,
}
impl std::fmt::Display for ErrorDelete {
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
			Self::WorksOnlyForDocument => f.write_str("this method works only on documents"),
			Self::WrongPath => f.write_str("the path of the item is incorrect"),
		}
	}
}
impl std::error::Error for ErrorDelete {}

#[cfg(feature = "server_bin")]
impl std::convert::From<ErrorDelete> for actix_web::HttpResponse {
	fn from(input: ErrorDelete) -> Self {
		let request_method = actix_web::http::Method::DELETE;
		match input {
			ErrorDelete::Conflict => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::CONFLICT,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorDelete::IfMatchNotFound => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorDelete::IfNoneMatch => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorDelete::InternalError => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorDelete::NotFound => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorDelete::WorksOnlyForDocument => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", input)),
				true,
			),
			ErrorDelete::WrongPath => crate::utils::build_http_json_response(
				&request_method,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", input)),
				true,
			),
		}
	}
}
