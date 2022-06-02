#[derive(Debug, PartialEq, Eq)]
pub enum GetError {
	Conflict {
		item_path: crate::item::ItemPath,
	},
	NotFound {
		item_path: crate::item::ItemPath,
	},
	IncorrectItemName {
		item_path: crate::item::ItemPath,
		error: String,
	},
	CanNotBeListed {
		item_path: crate::item::ItemPath,
	},
	NoIfMatch {
		item_path: crate::item::ItemPath,
		search: crate::item::Etag,
		found: crate::item::Etag,
	},
	IfNoneMatch {
		item_path: crate::item::ItemPath,
		search: crate::item::Etag,
		found: crate::item::Etag,
	},
	CanNotGetStorage,
	CanNotSerializeFile {
		item_path: crate::item::ItemPath,
		error: String,
	},
	IsSystemFile,
}
impl std::fmt::Display for GetError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Conflict { item_path } => f.write_fmt(format_args!("name conflict between folder and file on the path `{}`", item_path)),
			Self::NotFound { item_path } => f.write_fmt(format_args!("path not found : `{}`", item_path)),
			Self::IncorrectItemName { item_path, error } => f.write_fmt(format_args!("the path `{}` is incorrect, because {}", item_path, error)),
			Self::CanNotBeListed { item_path } => f.write_fmt(format_args!("the folder `{}` can not be listed", item_path)),
			Self::NoIfMatch { item_path, search, found } => f.write_fmt(format_args!("the requested `{}` etag (through `IfMatch`) for `{}` was not found, found `{}` instead", search, item_path, found)),
			Self::IfNoneMatch { item_path, search, found } => f.write_fmt(format_args!("the unwanted etag `{}` (through `IfNoneMatch`) for `{}` was matches with `{}`", search, item_path, found)),
			Self::CanNotGetStorage => f.write_str("can not get storage"),
			Self::CanNotSerializeFile { item_path, error } => f.write_fmt(format_args!("can not parse file `{}` because {}", item_path, error)),
			Self::IsSystemFile => f.write_str("this is a system file, that should not be server"),
		}
	}
}
impl std::error::Error for GetError {}
#[cfg(feature = "server_bin")]
impl crate::database::Error for GetError {
	fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse {
		match self {
			Self::Conflict { item_path } => {
				if item_path.starts_with("public/") {
					crate::database::build_http_json_response(
						origin,
						&actix_web::http::Method::GET,
						actix_web::http::StatusCode::NOT_FOUND,
						None,
						None,
						Some(format!("path not found : `{}`", item_path)),
						should_have_body,
					)
				} else {
					crate::database::build_http_json_response(
						origin,
						&actix_web::http::Method::GET,
						actix_web::http::StatusCode::CONFLICT,
						None,
						None,
						Some(format!("{}", self)),
						should_have_body,
					)
				}
			}
			Self::NotFound { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::IncorrectItemName { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::CanNotBeListed { item_path } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				None,
				Some(format!("path not found : `{}`", item_path)),
				should_have_body,
			),
			Self::NoIfMatch { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::IfNoneMatch { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::CanNotGetStorage => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::CanNotSerializeFile { .. } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				should_have_body,
			),
			Self::IsSystemFile => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				None,
				None,
				should_have_body,
			),
		}
	}
}
