#![allow(clippy::needless_return)]

mod client;
mod database;

pub use database::*;

#[derive(Debug, Clone, serde::Serialize)]
pub enum Item {
	Folder {
		etag: String,
		content: std::collections::HashMap<String, Box<Item>>,
	},
	Document {
		etag: String,
		content: Vec<u8>,
	},
}
impl Item {
	fn get_etag(&self) -> String {
		return match self {
			Self::Folder { etag, content: _ } => etag.clone(),
			Self::Document { etag, content: _ } => etag.clone(),
		};
	}
}

/*
TODO : Bearer tokens and access control
	* <module> string SHOULD be lower-case alphanumerical, other
		than the reserved word 'public'
	* <level> can be ':r' or ':rw'.

	<module> ':rw') any requests to paths relative to <storage_root>
					that start with '/' <module> '/' or
					'/public/' <module> '/',
	<module> ':r') any GET or HEAD requests to paths relative to
					<storage_root> that start with
					'/' <module> '/' or '/public/' <module> '/',
*/
