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
	CanNotSendEvent(std::sync::mpsc::SendError<crate::database::Event>, String),
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
			Self::CanNotSendEvent(_, _) => f.write_str("the save event can not be send"),
		}
	}
}
impl std::error::Error for ErrorPut {}

#[cfg(feature = "server_bin")]
impl ErrorPut {
	pub fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse {
		let request_method = actix_web::http::Method::PUT;
		match self {
			ErrorPut::Conflict => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::CONFLICT,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			ErrorPut::IfMatchNotFound => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			ErrorPut::IfNoneMatch => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			ErrorPut::InternalError => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			ErrorPut::NotFound => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			ErrorPut::NotModified => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::NOT_MODIFIED,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			ErrorPut::WorksOnlyForDocument => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			ErrorPut::WrongPath => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			ErrorPut::CanNotSendEvent(_, etag) => crate::database::build_http_json_response(
				origin,
				&request_method,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				Some(etag.clone()),
				Some(format!("{}", self)),
				should_have_body,
			),
		}
	}
}
