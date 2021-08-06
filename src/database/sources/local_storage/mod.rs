mod delete;
mod get;
mod put;

pub use delete::*;
pub use get::{get, GetError};
pub use put::*;

/// Store data in web browser's localStorage.
///
/// It is a local key-value database available in modern web browsers.
///
/// [More on MDN](https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage)
#[derive(Debug)]
pub struct LocalStorage {
	/// The prefix of the keys inside the localStorage.
	pub prefix: String,
}
impl crate::database::DataSource for LocalStorage {
	fn get(
		&self,
		path: &crate::item::ItemPath,
		if_match: &crate::item::Etag,
		if_none_match: &[&crate::item::Etag],
		get_content: bool,
	) -> Result<crate::item::Item, Box<dyn std::error::Error>> {
		match web_sys::window() {
			Some(window) => match window.local_storage() {
				Ok(Some(local_storage)) => get(
					&local_storage,
					&self.prefix,
					path,
					if_match,
					if_none_match,
					get_content,
				),
				Ok(None) => Err(Box::new(LocalStorageError::ThereIsNoLocalStorage)),
				Err(_) => Err(Box::new(LocalStorageError::CanNotGetLocalStorage)),
			},
			None => Err(Box::new(LocalStorageError::CanNotGetWindow)),
		}
	}

	fn put(
		&mut self,
		path: &crate::item::ItemPath,
		if_match: &crate::item::Etag,
		if_none_match: &[&crate::item::Etag],
		new_item: crate::item::Item,
	) -> crate::database::PutResult {
		match web_sys::window() {
			Some(window) => match window.local_storage() {
				Ok(Some(local_storage)) => put(
					&local_storage,
					&self.prefix,
					path,
					if_match,
					if_none_match,
					new_item,
				),
				Ok(None) => crate::database::PutResult::Err(Box::new(
					super::local_storage::LocalStorageError::ThereIsNoLocalStorage,
				)),
				Err(_) => crate::database::PutResult::Err(Box::new(
					super::local_storage::LocalStorageError::CanNotGetLocalStorage,
				)),
			},
			None => crate::database::PutResult::Err(Box::new(
				super::local_storage::LocalStorageError::CanNotGetWindow,
			)),
		}
	}

	fn delete(
		&mut self,
		path: &crate::item::ItemPath,
		if_match: &crate::item::Etag,
	) -> Result<crate::item::Etag, Box<dyn std::error::Error>> {
		match web_sys::window() {
			Some(window) => match window.local_storage() {
				Ok(Some(local_storage)) => delete(&local_storage, &self.prefix, path, if_match),
				Ok(None) => Err(Box::new(
					super::local_storage::LocalStorageError::ThereIsNoLocalStorage,
				)),
				Err(_) => Err(Box::new(
					super::local_storage::LocalStorageError::CanNotGetLocalStorage,
				)),
			},
			None => Err(Box::new(
				super::local_storage::LocalStorageError::CanNotGetWindow,
			)),
		}
	}
}

pub trait Storage {
	fn length(&self) -> Result<u32, wasm_bindgen::JsValue>;
	fn clear(&self) -> Result<(), wasm_bindgen::JsValue>;
	fn get_item(&self, key: &str) -> Result<Option<String>, wasm_bindgen::JsValue>;
	fn key(&self, index: u32) -> Result<Option<String>, wasm_bindgen::JsValue>;
	fn remove_item(&self, key: &str) -> Result<(), wasm_bindgen::JsValue>;
	fn set_item(&self, key: &str, value: &str) -> Result<(), wasm_bindgen::JsValue>;
}

#[cfg(feature = "server_local_storage")]
impl Storage for web_sys::Storage {
	fn length(&self) -> Result<u32, wasm_bindgen::JsValue> {
		web_sys::Storage::length(self)
	}
	fn clear(&self) -> Result<(), wasm_bindgen::JsValue> {
		web_sys::Storage::clear(self)
	}
	fn get_item(&self, key: &str) -> Result<Option<String>, wasm_bindgen::JsValue> {
		web_sys::Storage::get_item(self, key)
	}
	fn key(&self, index: u32) -> Result<Option<String>, wasm_bindgen::JsValue> {
		web_sys::Storage::key(self, index)
	}
	fn remove_item(&self, key: &str) -> Result<(), wasm_bindgen::JsValue> {
		web_sys::Storage::remove_item(self, key)
	}
	fn set_item(&self, key: &str, value: &str) -> Result<(), wasm_bindgen::JsValue> {
		web_sys::Storage::set_item(self, key, value)
	}
}

#[cfg(feature = "server_local_storage")]
#[derive(Debug, PartialEq, Eq)]
pub enum LocalStorageError {
	CanNotGetWindow,
	CanNotGetLocalStorage,
	ThereIsNoLocalStorage,
}
impl std::fmt::Display for LocalStorageError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::CanNotGetWindow => f.write_str("can not get window API"),
			Self::CanNotGetLocalStorage => f.write_str("can not get localStorage API"),
			Self::ThereIsNoLocalStorage => f.write_str("there is no localStorage available"),
		}
	}
}
impl std::error::Error for LocalStorageError {}
#[cfg(feature = "server_bin")]
impl crate::database::Error for LocalStorageError {
	// TODO : we have to find a way to change method
	fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse {
		match self {
			Self::CanNotGetWindow => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::CanNotGetLocalStorage => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::ThereIsNoLocalStorage => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::GET,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
		}
	}
}

#[derive(Debug)]
struct LocalStorageMock {
	content: std::cell::RefCell<std::collections::BTreeMap<String, String>>,
}
impl LocalStorageMock {
	#[allow(dead_code)]
	pub fn new() -> Self {
		LocalStorageMock {
			content: std::cell::RefCell::new(std::collections::BTreeMap::new()),
		}
	}
}
impl Storage for LocalStorageMock {
	fn length(&self) -> Result<u32, wasm_bindgen::JsValue> {
		Ok(self.content.borrow().len() as u32)
	}
	fn clear(&self) -> Result<(), wasm_bindgen::JsValue> {
		(*self.content.borrow_mut()).clear();
		Ok(())
	}
	fn get_item(&self, key: &str) -> Result<Option<String>, wasm_bindgen::JsValue> {
		Ok(self.content.borrow().get(key).map(String::clone))
	}
	fn key(&self, index: u32) -> Result<Option<String>, wasm_bindgen::JsValue> {
		Ok(self
			.content
			.borrow()
			.keys()
			.nth(index as usize)
			.map(String::clone))
	}
	fn remove_item(&self, key: &str) -> Result<(), wasm_bindgen::JsValue> {
		(*self.content.borrow_mut()).remove(key);
		Ok(())
	}
	fn set_item(&self, key: &str, value: &str) -> Result<(), wasm_bindgen::JsValue> {
		(*self.content.borrow_mut()).insert(String::from(key), String::from(value));
		Ok(())
	}
}
