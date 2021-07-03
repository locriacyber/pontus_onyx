pub struct RemoteStorage;
impl RemoteStorage {
	pub fn new() -> Self {
		todo!()
	}
}
impl RemoteStorage {
	pub fn logging(&mut self, _enable: bool) -> Self {
		todo!()
	}
	pub fn cache(&mut self, _conditions: Vec<CacheCondition>) -> Self {
		todo!()
	}
}
impl RemoteStorage {
	/// Register an event handler.
	pub fn on(
		&mut self,
		_event: EventName,
		_listener: std::sync::Arc<std::sync::Mutex<dyn FnMut(Event) + Send>>,
	) {
		todo!()
	}
	/// Connect to a remoteStorage server.
	///
	/// Discovers the WebFinger profile of the given user address and initiates the OAuth dance.
	///
	/// This method must be called *after* all required access has been claimed. When using the connect widget, it will call this method itself.
	///
	/// Special cases :
	///
	/// 1. If a bearer token is supplied as second argument, the OAuth dance will be skipped and the supplied token be used instead.
	///    This is useful outside of browser environments, where the token has been acquired in a different way.
	/// 2. If the Webfinger profile for the given user address doesn’t contain an auth URL,
	///    the library will assume that client and server have established authorization among themselves,
	///    which will omit bearer tokens in all requests later on.
	///    This is useful for example when using Kerberos and similar protocols.
	///
	/// # Arguments
	///
	/// - address : user address (user@host) to connect to.
	/// - token : a bearer token acquired beforehand
	///
	/// # Example
	///
	/// ```
	/// let mut remote = RemoteStorage::new();
	/// remote.connect("user@example.com");
	/// ```
	pub fn connect(&mut self, _address: &str, _token: Option<&str>) {
		todo!()
	}
	/// “Disconnect” from remote server to terminate current session.
	///
	/// This method clears all stored settings and deletes the entire local cache.
	///
	/// # Example
	///
	/// ```
	/// let mut remote = RemoteStorage::new();
	/// remote.connect("user@example.com");
	/// remote.disconnect();
	/// ```
	pub fn disconnect(&mut self) {
		todo!()
	}
	/// Start synchronization with remote storage, downloading and uploading any changes within the cached paths.
	///
	/// Please consider: local changes will attempt sync immediately, and remote changes should also be synced timely when using library defaults.
	/// So this is mostly useful for letting users sync manually, when pressing a sync button for example.
	/// This might feel safer to them sometimes, esp. when shifting between offline and online a lot.
	pub fn start_sync(&mut self) {
		todo!()
	}
	/// Stop the periodic synchronization.
	pub fn stop_sync(&mut self) {
		todo!()
	}
}
impl RemoteStorage {
	/// Set the value of the sync interval when application is in the foreground.
	///
	/// # Arguments
	///
	/// - value : sync interval in milliseconds (between 1000 and 3600000)
	pub fn set_sync_interval(&mut self, _value: usize) {
		todo!()
	}
	/// Set the value of the sync interval when the application is in the background.
	///
	/// # Arguments
	///
	/// - value : sync interval in milliseconds (between 1000 and 3600000)
	pub fn set_background_sync_interval(&mut self, _value: usize) {
		todo!()
	}
	/// Set the timeout for network requests.
	///
	/// # Arguments
	///
	/// - value : timeout in milliseconds
	pub fn set_request_timeout(&mut self, _value: usize) {
		todo!()
	}
	/// Set the OAuth key/ID for either GoogleDrive or Dropbox backend support.
	pub fn set_api_key(&mut self, _key: ApiKey) {
		todo!()
	}
}
impl RemoteStorage {
	/// Get the value of the sync interval when application is in the foreground, in milliseconds.
	pub fn get_sync_interval(&self) -> usize {
		todo!()
	}
	/// Get the value of the sync interval when application is in the background, in milliseconds.
	pub fn get_background_sync_interval(&self) -> usize {
		todo!()
	}
	/// Get the value of the current sync interval, in milliseconds.
	/// Can be background or foreground, custom or default.
	pub fn get_current_sync_interval(&self) -> usize {
		todo!()
	}
	/// Get the value of the current network request timeout, in milliseconds.
	pub fn get_request_timeout(&self) -> usize {
		todo!()
	}
}
impl RemoteStorage {
	/// This method enables you to quickly instantiate a BaseClient, which you can use to directly read and manipulate data in the connected storage account.
	///
	/// Please use this method only for debugging and development, and choose or create a data module for your app to use.
	///
	/// # Arguments
	///
	/// - path : the base directory of the BaseClient that will be returned (with a leading and a trailing slash)
	///
	/// # Returns
	///
	/// A client with the specified scope (category/base directory)
	///
	/// # Example
	///
	/// ```
	/// let remote = RemoteStorage::new();
	/// remote.scope("/pictures/").getListing("");
	/// remote.scope("/public/pictures/").getListing("");
	/// ```
	pub fn scope(&self, _path: &str) -> super::BaseClient {
		todo!()
	}
}
impl RemoteStorage {
	/// Initiate the OAuth authorization flow.
	///
	/// This function is called by custom storage backend implementations (e.g. Dropbox or Google Drive).
	///
	/// # Arguments
	///
	/// - auth_url : URL of the authorization endpoint
	/// - scope : access scope
	/// - client_id : client identifier (defaults to the origin of the redirectUri)
	pub fn authorize(&self, _auth_url: &str, _scope: &str, _client_id: &str) {
		todo!()
	}
}

pub enum CacheCondition {
	All,
	StartsWith(std::path::PathBuf),
}

pub enum EventName {
	/// Emitted when all features are loaded and the RS instance is ready
	Ready,
	/// Emitted when ready, but no storage connected (“anonymous mode”)
	NotConnected,
	/// Emitted when a remote storage has been connected
	Connected,
	/// Emitted after disconnect
	Disconnected,
	/// Emitted when an error occurs; receives an error object as argument
	Error(String),
	// /// Emitted when all features are loaded
	// FeaturesLoaded,
	/// Emitted before webfinger lookup
	Connecting,
	/// Emitted before redirecting to the authing server
	Authing,
	/// Emitted when a network request starts
	WireBusy,
	/// Emitted when a network request completes
	WireDone,
	/// Emitted when a single sync request has finished
	SyncRecDone,
	/// Emitted when all tasks of a sync have been completed and a new sync is scheduled
	SyncDone,
	/// Emitted once when a wire request fails for the first time, and `remote.online` is set to false
	NetworkOffline,
	/// Emitted once when a wire request succeeds for the first time after a failed one, and `remote.online` is set back to true
	NetworkOnline,
	/// Emitted when the sync interval changes
	SyncIntervalChange,
}

pub enum ApiKey {
	Dropbox(String),
	GoogleDrive(String),
}

pub enum Event {}
