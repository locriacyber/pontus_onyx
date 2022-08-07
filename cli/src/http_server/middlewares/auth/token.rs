use rand::seq::IteratorRandom;
use rand::Rng;

#[derive(Clone, Debug)]
pub struct OauthFormToken {
	ip: std::net::SocketAddr,
	forged: std::time::Instant,
	value: String,
}
impl OauthFormToken {
	pub fn new(ip: std::net::SocketAddr) -> Self {
		let forged = std::time::Instant::now();
		let mut value = String::new();

		let mut rng_limit = rand::thread_rng();
		for _ in 1..rng_limit.gen_range(32..64) {
			let mut rng_item = rand::thread_rng();
			value.push(
				crate::http_server::FORM_TOKEN_ALPHABET
					.chars()
					.choose(&mut rng_item)
					.unwrap(),
			);
		}

		Self { ip, forged, value }
	}

	/*
	pub fn get_ip(&self) -> &std::net::SocketAddr {
		&self.ip
	}

	pub fn get_forged(&self) -> &std::time::Instant {
		&self.forged
	}
	*/

	pub fn get_value(&self) -> &str {
		&self.value
	}

	pub fn has_expirated(&self) -> bool {
		(std::time::Instant::now() - self.forged) == std::time::Duration::from_secs(5 * 60)
	}

	pub fn should_be_cleaned(&self, ip: &std::net::SocketAddr) -> bool {
		if self.has_expirated() {
			return true;
		}

		if &self.ip == ip {
			return true;
		}

		return false;
	}
}
