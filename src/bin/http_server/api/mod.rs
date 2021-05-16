mod delete;
mod get;
mod head;
mod oauth;
mod options;
mod put;

pub use delete::delete_item;
pub use get::get_item;
pub use head::head_item;
pub use oauth::*;
pub use options::options_item;
pub use put::put_item;

fn convert_actix_if_match(request: &actix_web::web::HttpRequest) -> Vec<String> {
	let res: Result<actix_web::http::header::IfMatch, actix_web::error::ParseError> =
		actix_web::http::header::Header::parse(request);

	match res {
		Ok(res) => match res {
			actix_web::http::header::IfMatch::Any => vec![String::from("*")],
			actix_web::http::header::IfMatch::Items(items) => items
				.into_iter()
				.map(|etag| etag.tag().trim().to_string())
				.collect(),
		},
		Err(_) => vec![],
	}
}

fn convert_actix_if_none_match(request: &actix_web::web::HttpRequest) -> Vec<String> {
	let res: Result<actix_web::http::header::IfNoneMatch, actix_web::error::ParseError> =
		actix_web::http::header::Header::parse(request);

	match res {
		Ok(res) => match res {
			actix_web::http::header::IfNoneMatch::Any => vec![String::from("*")],
			actix_web::http::header::IfNoneMatch::Items(items) => items
				.into_iter()
				.map(|etag| etag.tag().trim().to_string())
				.collect(),
		},
		Err(_) => vec![],
	}
}
