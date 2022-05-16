use js_sys::Promise;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

lazy_static::lazy_static! {
	static ref ACCESS_TOKEN_REGEX: regex::Regex = regex::Regex::new("^#.*access_token=([^&]+).+$").unwrap();
}

const OAUTH_KEY: &str = "http://tools.ietf.org/html/rfc6749#section-4.2";

pub struct ClientRemote {
	webfinger_root_uri: String,
	username: String,
	scope: crate::scope::Scope,
	client_id: String,
	pub debug: bool,
	client: Option<Client>,
}
impl ClientRemote {
	pub async fn new(
		webfinger_root_uri: impl Into<String>,
		username: impl Into<String>,
		scope: crate::scope::Scope,
		client_id: impl Into<String>,
		debug: bool,
	) -> Result<Self, JsValue> {
		let webfinger_root_uri = webfinger_root_uri.into();
		let username = username.into();
		let client_id = client_id.into();

		////////////////////////

		let mut result = Self {
			webfinger_root_uri,
			username,
			scope,
			client_id,
			debug,
			client: None,
		};

		result.try_mount_saved_client().await?;

		Ok(result)
	}
}
impl ClientRemote {
	pub fn head(
		&self,
		path: &crate::item::ItemPath,
		etag: Option<crate::item::Etag>,
	) -> Result<Promise, JsValue> {
		match &self.client {
			Some(client) => client.head(path, etag),
			None => Err(JsValue::from_str("client is not connected")),
		}
	}
	pub fn get(
		&self,
		path: &crate::item::ItemPath,
		etag: Option<crate::item::Etag>,
	) -> Result<Promise, JsValue> {
		match &self.client {
			Some(client) => client.get(path, etag),
			None => Err(JsValue::from_str("client is not connected")),
		}
	}
	pub fn put(
		&self,
		path: &crate::item::ItemPath,
		document: &crate::item::Item,
	) -> Result<Promise, JsValue> {
		match &self.client {
			Some(client) => client.put(path, document),
			None => Err(JsValue::from_str("client is not connected")),
		}
	}
}
impl ClientRemote {
	fn generate_cookie_name_header(&self) -> String {
		let webfinger_root_uri_obj = self.webfinger_root_uri.parse::<http::uri::Uri>().unwrap();
		let client_id_uri_obj = self.client_id.parse::<http::uri::Uri>().unwrap();

		let cookie_name_header = format!(
			"{}|{}|{}|{}|",
			match client_id_uri_obj.port() {
				Some(port) => format!("{}:{}", client_id_uri_obj.host().unwrap(), port),
				None => String::from(client_id_uri_obj.host().unwrap()),
			},
			self.username,
			match webfinger_root_uri_obj.port() {
				Some(port) => format!("{}:{}", webfinger_root_uri_obj.host().unwrap(), port),
				None => String::from(webfinger_root_uri_obj.host().unwrap()),
			},
			self.scope
		);
		let cookie_name_header =
			pct_str::PctString::encode(cookie_name_header.chars(), pct_str::URIReserved);

		cookie_name_header.to_string()
	}
	async fn try_get_webfinger_data(&self) -> Result<WebfingerResponse, JsValue> {
		let mut opts = web_sys::RequestInit::new();
		opts.method("GET");
		opts.mode(web_sys::RequestMode::Cors);

		let webfinger_uri = self
			.webfinger_root_uri
			.strip_suffix('/')
			.unwrap_or(&self.webfinger_root_uri);

		let webfinger_root_uri_obj = self.webfinger_root_uri.parse::<http::uri::Uri>().unwrap();

		let url = format!(
			"{webfinger_uri}/.well-known/webfinger?resource=acct:{}@{}",
			self.username,
			webfinger_root_uri_obj.host().unwrap()
		);

		let request = web_sys::Request::new_with_str_and_init(&url, &opts)?;

		let window = web_sys::window().ok_or_else(|| JsValue::from_str("window not found"))?;

		if self.debug {
			web_sys::console::log_1(
				&format!("pontus-onyx-client-debug: trying to fetch GET {url}").into(),
			);
		}

		let resp_value =
			wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;
		let resp: web_sys::Response = resp_value.dyn_into()?;
		let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;
		let response: WebfingerResponse = json.into_serde().unwrap();

		Ok(response)
	}
	async fn try_mount_saved_client(&mut self) -> Result<bool, JsValue> {
		let window = web_sys::window().ok_or_else(|| JsValue::from_str("window not found"))?;
		let document = window
			.document()
			.ok_or_else(|| JsValue::from_str("document not found"))?;
		let document = document
			.dyn_ref::<web_sys::HtmlDocument>()
			.ok_or_else(|| JsValue::from_str("document can not be casted into HtmlDocument"))?;

		let hash = window.location().hash()?;

		let access_token = if hash.contains("token_type") && ACCESS_TOKEN_REGEX.is_match(&hash) {
			if let Some(matches) = ACCESS_TOKEN_REGEX.captures_iter(&hash).next() {
				matches.get(1).map(|access_token| {
					let access_token = access_token.as_str();

					if self.debug {
						web_sys::console::log_1(&format!("pontus-onyx-client-debug: found token in URL hash : {access_token}").into());
					}

					pct_str::PctString::new(access_token)
						.unwrap()
						.decode()
				})
			} else {
				None
			}
		} else {
			None
		};

		let access_token = if cfg!(feature = "client_lib_cookies") {
			match access_token {
				Some(access_token) => {
					// hide token from URL
					window.history()?.replace_state_with_url(
						&String::new().into(),
						"",
						Some("/"),
					)?;

					if self.debug {
						web_sys::console::log_1(
							&format!(
								"pontus-onyx-client-debug: trying to set cookie {}access_token",
								self.generate_cookie_name_header()
							)
							.into(),
						);
					}

					document
						.set_cookie(&format!(
							"{}access_token={}; Secure",
							self.generate_cookie_name_header(),
							pct_str::PctString::encode(access_token.chars(), pct_str::URIReserved)
						))
						.unwrap();

					Some(access_token)
				}
				None => {
					let document = window
						.document()
						.ok_or_else(|| JsValue::from_str("document not found"))?;
					let document = document.dyn_ref::<web_sys::HtmlDocument>().unwrap();

					let all_cookies = document.cookie()?;

					let mut access_token = None;
					for cookie in all_cookies.split(';') {
						let mut iter = cookie.split('=');
						let name = iter.next().map(str::trim);
						let value = iter
							.next()
							.map(|res| pct_str::PctString::new(res.trim()).unwrap().decode());

						if let Some(name) = name {
							if name == format!("{}access_token", self.generate_cookie_name_header())
							{
								if self.debug {
									web_sys::console::log_1(
										&format!(
											"pontus-onyx-client-debug: found cookie {}access_token",
											self.generate_cookie_name_header()
										)
										.into(),
									);
								}

								access_token = value;
								break;
							}
						}
					}

					access_token
				}
			}
		} else {
			if self.debug {
				web_sys::console::log_1(
					&"pontus-onyx-client-debug: cookies storage disabled".into(),
				);
			}

			access_token
		};

		match access_token {
			Some(access_token) => {
				let webfinger = self.try_get_webfinger_data().await?;

				let server_path = webfinger.links.get(0).map(|link| link.href.clone());

				match server_path {
					Some(server_path) => {
						let server_path = if !server_path.ends_with('/') {
							format!("{server_path}/")
						} else {
							server_path
						};

						if self.debug {
							web_sys::console::log_1(&format!("pontus-onyx-client-debug: found server path in webfinger response : {server_path}").into());
						}

						let mut opts = web_sys::RequestInit::new();
						opts.method("HEAD");
						opts.mode(web_sys::RequestMode::Cors);

						let full_path = format!("{}{}/", server_path, self.scope.module);

						if self.debug {
							web_sys::console::log_1(
								&format!(
									"pontus-onyx-client-debug: trying to fetch HEAD {full_path}"
								)
								.into(),
							);
						}

						let request =
							web_sys::Request::new_with_str_and_init(&full_path, &opts).unwrap();
						request
							.headers()
							.set("Authorization", &format!("Bearer {}", access_token))?;

						let window = web_sys::window().ok_or("window not found")?;

						let root_head = wasm_bindgen_futures::JsFuture::from(Promise::new(
							&mut |resolve, reject| {
								let debug = self.debug;
								let full_path_for_main = full_path.clone();
								let process_callback =
									Closure::once(Box::new(move |resp: JsValue| {
										let resp: web_sys::Response = resp.dyn_into().unwrap();

										if resp.ok() {
											if debug {
												web_sys::console::log_1(&format!("pontus-onyx-client-debug: HEAD {full_path_for_main} response is OK").into());
											}

											resolve
												.call1(&JsValue::NULL, &JsValue::from_bool(true))
												.unwrap();
										} else {
											if debug {
												web_sys::console::log_1(&format!("pontus-onyx-client-debug: HEAD {full_path_for_main} response is NOT OK").into());
											}

											resolve
												.call1(&JsValue::NULL, &JsValue::from_bool(false))
												.unwrap();
										}
									}) as Box<dyn FnOnce(JsValue)>);

								let err_callback = Closure::wrap(Box::new(move |err: JsValue| {
									reject
										.call1(&JsValue::NULL, &format!("{:?}", err).into())
										.unwrap();
								}) as Box<dyn FnMut(JsValue)>);

								window
									.fetch_with_request(&request)
									.then(&process_callback)
									.catch(&err_callback);

								process_callback.forget();
								err_callback.forget();
							},
						))
						.await?;

						let root_head = root_head.as_bool();

						if let Some(true) = root_head {
							self.client = Some(Client {
								access_token,
								server_path,
								debug: self.debug,
							});

							Ok(true)
						} else {
							Ok(false)
						}
					}
					None => Err(JsValue::from_str(
						"can not find `links` content in webfinger reponse of the server",
					)),
				}
			}
			None => Ok(false),
		}
	}
}
impl ClientRemote {
	pub async fn show_connect_overlay(
		&self,
		absolute_uri_handle: impl AsRef<str>,
	) -> Result<(), JsValue> {
		let absolute_uri_handle = absolute_uri_handle.as_ref();
		let webfinger = self.try_get_webfinger_data().await?;

		match webfinger.links.get(0) {
			Some(link) => {
				let window =
					web_sys::window().ok_or_else(|| JsValue::from_str("window not found"))?;
				let document = window
					.document()
					.ok_or_else(|| JsValue::from_str("document not found"))?;

				let res = if document.get_element_by_id("pontus_onyx_oauth_next_window").is_none() {

					let location = window.location();
					let oauth_origin = link.properties.get(OAUTH_KEY).unwrap().as_ref().unwrap();
					let oauth_path = format!(
						"{oauth_origin}?redirect_uri={}&scope={}&client_id={}&response_type={}",
						pct_str::PctString::encode(
							format!(
								"{}//{}{}",
								location.protocol()?,
								location.host()?,
								absolute_uri_handle
							)
							.chars(),
							pct_str::URIReserved
						), // TODO : change to base url (no page name, or its arguments)
						pct_str::PctString::encode(
							format!("{}", self.scope).chars(),
							pct_str::URIReserved
						),
						pct_str::PctString::encode(self.client_id.chars(), pct_str::URIReserved),
						pct_str::PctString::encode("token".chars(), pct_str::URIReserved),
					);

					// location.set_href(&oauth_path).unwrap();

					let next_window = document.create_element("div")?;
					next_window.set_attribute("id", "pontus_onyx_oauth_next_window")?;
					let next_window = next_window.dyn_ref::<web_sys::HtmlElement>().unwrap();

					next_window
						.style()
						.set_property("border", "5px solid #FF4B03")?;
					next_window.style().set_property("background", "white")?;
					next_window.style().set_property("color", "black")?;
					next_window.style().set_property("padding", "1em")?;
					next_window.style().set_property("position", "absolute")?;
					next_window.style().set_property("width", "66%")?;
					next_window.style().set_property("left", "17%")?;
					next_window.style().set_property("top", "30px")?;
					next_window.style().set_property("opacity", "0.8")?;
					next_window.style().set_property("display", "flex")?;
					next_window.style().set_property("flex-direction", "column")?;
					next_window.style().set_property("align-items", "stretch")?;
					next_window.style().set_property("align-content", "stretch")?;
					next_window.style().set_property("gap", "1em")?;

					let abort = document.create_element("button")?;
					let abort = abort.dyn_ref::<web_sys::HtmlElement>().unwrap();
					abort.style().set_property("border", "2px solid #FF4B03")?;
					abort.style().set_property("background", "white")?;
					abort.style().set_property("cursor", "pointer")?;
					abort.style().set_property("font-weight", "bold")?;
					abort.style().set_property("padding", "1em 0em")?;
					abort.style().set_property("color", "black")?;
					abort.set_inner_html("❌ Abort");
					let close_next_window = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
						if let Some(window) = web_sys::window() {
							if let Some(document) = window.document() {
								if let Some(body) = document.body() {
									if let Some(node) =
										document.get_element_by_id("pontus_onyx_oauth_next_window")
									{
										body.remove_child(&node).ok();
									}
								}
							}
						}
					})
						as Box<dyn FnMut()>);
					abort.set_onclick(Some(close_next_window.as_ref().unchecked_ref()));
					close_next_window.forget();

					next_window.append_child(abort)?;

					let dummy = document.create_element("div")?;
					dummy.set_inner_html(include_str!("../../static/remoteStorage.svg"));
					let dummy = dummy.dyn_ref::<web_sys::HtmlElement>().unwrap();

					let svg = dummy.first_element_child();
					if let Some(svg) = svg {
						svg.set_attribute("width", "100%")?;
						svg.set_attribute("height", "75")?;
						next_window.append_child(&svg)?;
					}

					let mut html = String::from("You will be temporary redirected to<br>\n");
					html += &format!(r#"<a href="{}">{}</a><br>"#, oauth_path, oauth_origin);
					html += "\nin order to authenticate<br>\n";
					html += "on the requested server,<br>\n";
					html += "and then bring back to this app<br>\n";
					html += "with some credentials";

					let explain = document.create_element("p")?;
					let explain = explain.dyn_ref::<web_sys::HtmlElement>().unwrap();
					explain.style().set_property("text-align", "center")?;
					explain.style().set_property("color", "black")?;
					explain.set_inner_html(&html);

					next_window.append_child(&explain)?;

					let a_next = document.create_element("a")?;
					let a_next = a_next.dyn_ref::<web_sys::HtmlElement>().unwrap();
					a_next.set_attribute("href", &oauth_path)?;
					a_next.style().set_property("display", "block")?;

					let button_next = document.create_element("button")?;
					let button_next = button_next.dyn_ref::<web_sys::HtmlElement>().unwrap();
					button_next.set_inner_html("Next ➡️");
					button_next
						.style()
						.set_property("border", "2px solid black")?;
					button_next.style().set_property("background", "#FF4B03")?;
					button_next.style().set_property("color", "black")?;
					button_next.style().set_property("cursor", "pointer")?;
					button_next.style().set_property("font-weight", "bold")?;
					button_next.style().set_property("width", "100%")?;
					button_next.style().set_property("padding", "1em 0em")?;
					a_next.append_child(button_next)?;

					next_window.append_child(&a_next)?;

					document.body().unwrap().append_child(next_window)?;

					// TODO : automatic redirection ?

					Ok(())
				} else {
					Err(JsValue::from_str(
						"the id `pontus_onyx_oauth_next_window` already exists in the window so the overlay should be probably already displayed",
					))
				};

				window.scroll_to_with_x_and_y(0.0, 0.0);

				res
			}
			None => Err(JsValue::from_str(
				"can not find `links` content in webfinger response of the server",
			)),
		}
	}
}
impl ClientRemote {
	pub fn is_connected(&self) -> bool {
		self.client.is_some()
	}
}

pub struct Client {
	server_path: String,
	access_token: String,
	pub debug: bool,
}
impl Client {
	pub fn head(
		&self,
		path: &crate::item::ItemPath,
		etag: Option<crate::item::Etag>,
	) -> Result<Promise, JsValue> {
		let mut opts = web_sys::RequestInit::new();
		opts.method("HEAD");
		opts.mode(web_sys::RequestMode::Cors);

		let full_path = format!("{}{}", self.server_path, path);

		let request = web_sys::Request::new_with_str_and_init(&full_path, &opts).unwrap();
		request
			.headers()
			.set("Authorization", &format!("Bearer {}", self.access_token))?;
		if let Some(etag) = etag {
			request.headers().set("If-Match", &etag.to_string())?;
		}

		let window = web_sys::window().ok_or("window not found")?;

		if self.debug {
			web_sys::console::log_1(
				&format!("pontus-onyx-client-debug: trying to fetch HEAD {full_path} response")
					.into(),
			);
		}

		let is_folder = path.is_folder(); // TODO

		Ok(Promise::new(&mut |resolve, reject| {
			let reject = std::sync::Arc::new(reject);

			let debug = self.debug;
			let full_path_for_main = full_path.clone();
			let reject_for_main = reject.clone();
			let process_callback = Closure::once(Box::new(move |resp: JsValue| {
				let resp: web_sys::Response = resp.dyn_into().unwrap();

				if resp.ok() {
					let headers = resp.headers();

					if debug {
						web_sys::console::log_1(&format!("pontus-onyx-client-debug: server response for HEAD {full_path_for_main} is OK").into());
					}

					let etag = headers.get("etag");
					if etag.is_err() {
						reject_for_main
							.call1(
								&JsValue::NULL,
								&JsValue::from_str(
									"can not get `Etag` header from server response",
								),
							)
							.unwrap();
					}
					let etag = etag.unwrap();
					if etag.is_none() {
						reject_for_main
							.call1(
								&JsValue::NULL,
								&JsValue::from_str("missing `Etag` header from server response"),
							)
							.unwrap();
					}
					let etag = etag.unwrap();

					let content_type = headers.get("content-type");
					if content_type.is_err() {
						reject_for_main
							.call1(
								&JsValue::NULL,
								&JsValue::from_str(
									"can not get `Content-Type` header from server response",
								),
							)
							.unwrap();
					}
					let content_type = content_type.unwrap();
					if content_type.is_none() {
						reject_for_main
							.call1(
								&JsValue::NULL,
								&JsValue::from_str(
									"missing `Content-Type` header from server response",
								),
							)
							.unwrap();
					}
					let content_type = content_type.unwrap();

					resolve
						.call1(
							&JsValue::NULL,
							&JsValue::from_serde(&crate::item::Item::Document {
								etag: etag.into(),
								content: None,
								content_type: content_type.into(),
								last_modified: chrono::Utc::now(), // TODO
							})
							.unwrap(),
						)
						.unwrap();
				} else {
					reject_for_main
						.call1(
							&JsValue::NULL,
							&JsValue::from_str(&format!(
								"error {} when access to database",
								resp.status()
							)),
						)
						.unwrap();
				}
			}) as Box<dyn FnOnce(JsValue)>);

			let err_callback = Closure::wrap(Box::new(move |err: JsValue| {
				reject
					.call1(&JsValue::NULL, &format!("{:?}", err).into())
					.unwrap();
			}) as Box<dyn FnMut(JsValue)>);

			window
				.fetch_with_request(&request)
				.then(&process_callback)
				.catch(&err_callback);

			process_callback.forget();
			err_callback.forget();
		}))
	}
	pub fn get(
		&self,
		path: &crate::item::ItemPath,
		etag: Option<crate::item::Etag>,
	) -> Result<Promise, JsValue> {
		let mut opts = web_sys::RequestInit::new();
		opts.method("GET");
		opts.mode(web_sys::RequestMode::Cors);

		let full_path = format!("{}{}", self.server_path, path);

		let request = web_sys::Request::new_with_str_and_init(&full_path, &opts).unwrap();
		request
			.headers()
			.set("Authorization", &format!("Bearer {}", self.access_token))?;
		if let Some(etag) = etag {
			request.headers().set("If-Match", &etag.to_string())?;
		}

		let window = web_sys::window().ok_or("window not found")?;

		if self.debug {
			web_sys::console::log_1(
				&format!("pontus-onyx-client-debug: trying to fetch GET {full_path} response")
					.into(),
			);
		}

		let is_folder = path.is_folder();

		Ok(Promise::new(&mut |resolve, reject| {
			let reject = std::sync::Arc::new(reject);

			let debug = self.debug;
			let full_path_for_main = full_path.clone();
			let reject_for_main = reject.clone();
			let process_callback = Closure::once(Box::new(move |resp: JsValue| {
				let resp: web_sys::Response = resp.dyn_into().unwrap();

				if resp.ok() {
					let headers = resp.headers();

					if debug {
						web_sys::console::log_1(&format!("pontus-onyx-client-debug: server response for GET {full_path_for_main} is OK").into());
					}

					let etag = headers.get("etag");
					if etag.is_err() {
						reject_for_main
							.call1(
								&JsValue::NULL,
								&JsValue::from_str(
									"can not get `Etag` header from server response",
								),
							)
							.unwrap();
					}
					let etag = etag.unwrap();
					if etag.is_none() {
						reject_for_main
							.call1(
								&JsValue::NULL,
								&JsValue::from_str("missing `Etag` header from server response"),
							)
							.unwrap();
					}
					let etag = etag.unwrap();

					let content_type = headers.get("content-type");
					if content_type.is_err() {
						reject_for_main
							.call1(
								&JsValue::NULL,
								&JsValue::from_str(
									"can not get `Content-Type` header from server response",
								),
							)
							.unwrap();
					}
					let content_type = content_type.unwrap();
					if content_type.is_none() {
						reject_for_main
							.call1(
								&JsValue::NULL,
								&JsValue::from_str(
									"missing `Content-Type` header from server response",
								),
							)
							.unwrap();
					}
					let content_type = content_type.unwrap();

					let body_process = Closure::once(Box::new(move |body: JsValue| {
						if is_folder {
							todo!()
						} else {
							let body = js_sys::ArrayBuffer::from(body);
							let body = js_sys::DataView::new(
								&body,
								0,
								body.byte_length().try_into().unwrap(),
							);

							let mut buffer = vec![];
							for i in 0..body.byte_length() {
								buffer.push(body.get_uint8(i));
							}

							resolve
								.call1(
									&JsValue::NULL,
									&JsValue::from_serde(&crate::item::Item::Document {
										etag: etag.into(),
										content: Some(buffer),
										content_type: content_type.into(),
										last_modified: chrono::Utc::now(), // TODO
									})
									.unwrap(),
								)
								.unwrap();
						}
					}) as Box<dyn FnOnce(JsValue)>);

					let body_err = Closure::wrap(Box::new(move |err: JsValue| {
						reject_for_main
							.call1(&JsValue::NULL, &format!("{:?}", err).into())
							.unwrap();
					}) as Box<dyn FnMut(JsValue)>);

					resp.array_buffer()
						.unwrap()
						.then(&body_process)
						.catch(&body_err);

					body_process.forget();
					body_err.forget();
				} else if resp.status() == 404 {
					reject_for_main
						.call1(
							&JsValue::NULL,
							&JsValue::from_str("document does not exists yet in database"),
						)
						.unwrap();
				} else {
					reject_for_main
						.call1(
							&JsValue::NULL,
							&JsValue::from_str(&format!(
								"error {} when access to database",
								resp.status()
							)),
						)
						.unwrap();
				}
			}) as Box<dyn FnOnce(JsValue)>);

			let err_callback = Closure::wrap(Box::new(move |err: JsValue| {
				reject
					.call1(&JsValue::NULL, &format!("{:?}", err).into())
					.unwrap();
			}) as Box<dyn FnMut(JsValue)>);

			window
				.fetch_with_request(&request)
				.then(&process_callback)
				.catch(&err_callback);

			process_callback.forget();
			err_callback.forget();
		}))
	}
	pub fn put(
		&self,
		path: &crate::item::ItemPath,
		document: &crate::item::Item,
	) -> Result<Promise, JsValue> {
		if let crate::item::Item::Document {
			etag,
			content_type,
			content,
			last_modified: _,
		} = document
		{
			if let Some(content) = content {
				let mut opts = web_sys::RequestInit::new();
				opts.method("PUT");
				opts.body(Some(&js_sys::Uint8Array::from(content.as_slice())));
				opts.mode(web_sys::RequestMode::Cors);

				let full_path = format!("{}{}", self.server_path, path);

				let request = web_sys::Request::new_with_str_and_init(&full_path, &opts).unwrap();
				request
					.headers()
					.set("Authorization", &format!("Bearer {}", self.access_token))
					.unwrap();
				request
					.headers()
					.set("Content-Type", &format!("{}", content_type))
					.unwrap();

				if !etag.is_empty() {
					request
						.headers()
						.set("If-Match", &format!("{}", etag))
						.unwrap();
				}

				let window = web_sys::window().ok_or("window not found")?;

				if self.debug {
					web_sys::console::log_1(
						&format!("pontus-onyx-client-debug: trying to PUT to {full_path}").into(),
					);
				}

				Ok(Promise::new(&mut |resolve, reject| {
					let debug = self.debug;
					let full_path_for_main = full_path.clone();
					let reject_for_main = reject.clone();
					let process_callback = Closure::once(Box::new(move |resp: JsValue| {
						let resp: web_sys::Response = resp.dyn_into().unwrap();

						if resp.ok() {
							let headers = resp.headers();

							if debug {
								web_sys::console::log_1(&format!("pontus-onyx-client-debug: server response for PUT {full_path_for_main} is OK").into());
							}

							let etag = headers.get("etag");
							if etag.is_err() {
								reject_for_main
									.call1(
										&JsValue::NULL,
										&JsValue::from_str(
											"can not get `Etag` header from server response",
										),
									)
									.unwrap();
							}
							let etag = etag.unwrap();

							let content_type = headers.get("content-type");
							if content_type.is_err() {
								reject_for_main
									.call1(
										&JsValue::NULL,
										&JsValue::from_str(
											"can not get `Content-Type` header from server response",
										),
									)
									.unwrap();
							}
							let content_type = content_type.unwrap();
							if content_type.is_none() {
								reject_for_main
									.call1(
										&JsValue::NULL,
										&JsValue::from_str(
											"missing `Content-Type` header from server response",
										),
									)
									.unwrap();
							}
							let content_type = content_type.unwrap();

							resolve
								.call1(
									&JsValue::NULL,
									&JsValue::from_serde(&crate::item::Item::Document {
										etag: etag.unwrap_or_default().into(),
										content: None,
										content_type: content_type.into(),
										last_modified: chrono::Utc::now(), // TODO
									})
									.unwrap(),
								)
								.unwrap();
						} else {
							reject_for_main
								.call1(
									&JsValue::NULL,
									&JsValue::from_str(&format!(
										"error {} when access to database",
										resp.status()
									)),
								)
								.unwrap();
						}
					}) as Box<dyn FnOnce(JsValue)>);

					let err_callback = Closure::wrap(Box::new(move |err: JsValue| {
						reject
							.call1(&JsValue::NULL, &format!("{:?}", err).into())
							.unwrap();
					}) as Box<dyn FnMut(JsValue)>);

					window
						.fetch_with_request(&request)
						.then(&process_callback)
						.catch(&err_callback);

					process_callback.forget();
					err_callback.forget();
				}))
			} else {
				Err(JsValue::from("content of document is empty"))
			}
		} else {
			Err(JsValue::from("we can only put Item::Document to servers"))
		}
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct WebfingerResponse {
	links: Vec<Link>,
}
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct Link {
	href: String,
	properties: std::collections::HashMap<String, Option<String>>,
}
