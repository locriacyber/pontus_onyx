#[derive(Debug, PartialEq)]
pub enum DeleteError {
	GetError(super::super::GetError),
	DoesNotWorksForFolders,
	CanNotDelete {
		item_path: crate::item::ItemPath,
		error: String,
	},
	CanNotReadFile {
		item_path: crate::item::ItemPath,
		error: String,
	},
	CanNotWriteFile {
		item_path: crate::item::ItemPath,
		error: String,
	},
	CanNotSerializeFile {
		item_path: crate::item::ItemPath,
		error: String,
	},
	CanNotDeserializeFile {
		item_path: crate::item::ItemPath,
		error: String,
	},
}
impl std::fmt::Display for DeleteError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::GetError(get_error) => std::fmt::Display::fmt(get_error, f),
			Self::DoesNotWorksForFolders => f.write_str("this method does not works for folders"),
			Self::CanNotDelete { item_path, error } => f.write_fmt(format_args!(
				"can not delete file `{}` because : {}",
				item_path, error
			)),
			Self::CanNotReadFile { item_path, error } => f.write_fmt(format_args!(
				"can not read file `{}` because : {}",
				item_path, error
			)),
			Self::CanNotWriteFile { item_path, error } => f.write_fmt(format_args!(
				"can not write file `{}` because : {}",
				item_path, error
			)),
			Self::CanNotSerializeFile { item_path, error } => f.write_fmt(format_args!(
				"can not serialize file `{}` because : {}",
				item_path, error
			)),
			Self::CanNotDeserializeFile { item_path, error } => f.write_fmt(format_args!(
				"can not deserialize file `{}` because : {}",
				item_path, error
			)),
		}
	}
}
impl std::error::Error for DeleteError {}
#[cfg(feature = "server")]
impl crate::database::Error for DeleteError {
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
			Self::CanNotDelete {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				should_have_body,
			),
			Self::CanNotReadFile {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				should_have_body,
			),
			Self::CanNotWriteFile {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				should_have_body,
			),
			Self::CanNotSerializeFile {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				should_have_body,
			),
			Self::CanNotDeserializeFile {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
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
