pub struct Hsts {
	pub enable: bool,
}

impl<S, B> actix_web::dev::Transform<S, actix_web::dev::ServiceRequest> for Hsts
where
	S: actix_web::dev::Service<
		actix_web::dev::ServiceRequest,
		Response = actix_web::dev::ServiceResponse<B>,
		Error = actix_web::Error,
	>,
	S::Future: 'static,
	B: 'static,
{
	type Response = actix_web::dev::ServiceResponse<B>;
	type Error = actix_web::Error;
	type InitError = ();
	type Transform = HstsMiddleware<S>;
	type Future = futures::future::Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		futures::future::ok(Self::Transform {
			service,
			enable: self.enable,
		})
	}
}

pub struct HstsMiddleware<S> {
	service: S,
	enable: bool,
}

impl<S, B> actix_web::dev::Service<actix_web::dev::ServiceRequest> for HstsMiddleware<S>
where
	S: actix_web::dev::Service<
		actix_web::dev::ServiceRequest,
		Response = actix_web::dev::ServiceResponse<B>,
		Error = actix_web::Error,
	>,
	S::Future: 'static,
	B: 'static,
{
	type Response = actix_web::dev::ServiceResponse<B>;
	type Error = actix_web::Error;
	type Future =
		futures_util::future::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

	actix_web::dev::forward_ready!(service);

	fn call(&self, service_request: actix_web::dev::ServiceRequest) -> Self::Future {
		let future = self.service.call(service_request);
		let enable = self.enable;
		Box::pin(async move {
			future.await.map(|mut response| {
				if enable {
					response.response_mut().head_mut().headers.insert(
						actix_web::http::header::STRICT_TRANSPORT_SECURITY,
						actix_web::http::header::HeaderValue::from_str("max-age=31536000").unwrap(),
					);
				}

				response
			})
		})
	}
}
