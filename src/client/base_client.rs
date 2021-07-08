/// A `BaseClient` instance is the main endpoint you will use for interacting with a connected storage :
/// - listing documents,
/// - reading documents,
/// - creating documents,
/// - updating documents,
/// - deleting documents
/// as well as handling incoming changes.
///
/// Base clients are usually used in data modules, which are loaded with two `BaseClient` instances by default :
/// one for private and one for public documents.
///
/// However, you can also instantiate a BaseClient outside of a data module using the `remoteStorage.scope()` function.
/// Similarly, you can create a new scoped client within another client, using the `BaseClient`’s own `scope()`.
///
/// # Caching logic for read operations
///
/// All functions requesting/reading data will immediately return data from the local store, as long as it is reasonably up-to-date.
/// The default maximum age of requested data is two times the periodic sync interval (10 seconds by default).
///
/// However, you can adjust this behavior by using the `max_age` argument with any of these functions,
/// thereby changing the maximum age or removing the requirement entirely.
///
/// - If the `max_age` requirement is set, and the last sync request for the path is further in the past than the maximum age given,
///   the folder will first be checked for changes on the remote, and then the promise will be fulfilled with the up-to-date document or listing.
/// - If the `max_age` requirement is set, and cannot be met because of network problems, the promise will be rejected.
/// - If the `max_age` requirement is set to false, or the library is in offline mode, or no remote storage is connected (a.k.a. “anonymous mode”),
///   the promise will always be fulfilled with data from the local store.
///
/// ## Hint
///
/// If caching for the folder is turned off, none of this applies and data will always be requested from the remote store directly.
pub struct BaseClient;
impl BaseClient {
	pub fn new() -> Self {
		todo!()
	}

	/// Instantiate a new client, scoped to a subpath of the current client’s path.
	///
	/// # Arguments
	///
	/// - path : the path to scope the new client to.
	///
	/// # Returns
	///
	/// A new client operating on a subpath of the current base path.
	pub fn scope(&self, _path: &std::path::Path) -> Self {
		todo!()
	}

	/// Set caching strategy for a given path and its children.
	///
	/// # Arguments
	///
	/// - path : path to cache
	/// - strategy : caching strategy.
	pub fn cache(&self, _path: &std::path::Path, _strategy: CacheStrategy) -> Self {
		todo!()
	}

	/// Get a list of child nodes below a given path.
	///
	/// # Arguments
	///
	/// - path : the path to query. It MUST end with a forward slash.
	/// - max_age : the maximum age of cached listing in milliseconds. See Caching logic for read operations.
	///
	/// # Returns
	///
	/// The folder listing is returned as a JSON object, with the root keys representing the pathnames of child nodes.
	/// Keys ending in a forward slash represent folder nodes (subdirectories), while all other keys represent data nodes (files/objects).
	///
	/// Data node information contains the item’s ETag, content type and -length.
	///
	/// # Example
	///
	/// ```
	/// async {
	///     let client = pontus_onyx::client::BaseClient::new();
	///     let listing = client.get_listing(&std::path::PathBuf::from(""), None).await;
	/// };
	/// ```
	pub async fn get_listing(
		&self,
		_path: &std::path::Path,
		_max_age: Option<std::time::Duration>,
	) -> Box<dyn std::future::Future<Output = Vec<crate::Item>>> {
		todo!()
	}

	/// Get all objects directly below a given path.
	///
	/// # Arguments
	///
	/// - path : path to the folder. Must end in a forward slash.
	/// - max_age : the maximum age of cached objects in milliseconds. See Caching logic for read operations.
	///
	/// # Returns
	///
	/// For items that are not JSON-stringified objects (e.g. stored using storeFile instead of storeObject), the object’s value is filled in with true.
	pub fn get_all(
		&self,
		_path: &std::path::Path,
		_max_age: Option<std::time::Duration>,
	) -> Box<dyn std::future::Future<Output = Vec<crate::Item>>> {
		todo!()
	}

	/// Get a JSON object from the given path.
	///
	/// # Arguments
	///
	/// - path : relative path from the module root (without leading slash).
	/// - max_age : the maximum age of cached object in milliseconds. See Caching logic for read operations.
	///
	/// # Example
	///
	/// ```
	/// async {
	///     let client = pontus_onyx::client::BaseClient::new();
	///     let listing = client.get_object(&std::path::PathBuf::from("/path/to/object"), None).await;
	/// };
	/// ```
	pub async fn get_object(
		&self,
		_path: &std::path::Path,
		_max_age: Option<std::time::Duration>,
	) -> Box<dyn std::future::Future<Output = Result<serde_json::Value, Box<dyn std::any::Any>>>> {
		todo!()
	}

	/// Get the file at the given path. A file is raw data, as opposed to a JSON object (use `getObject()` for that).
	///
	/// # Arguments
	///
	/// - path : relative path from the module root (without leading slash).
	/// - max_age : the maximum age of the cached file in milliseconds. See Caching logic for read operations.
	///
	/// # Example
	///
	/// ```
	/// async {
	///     let client = pontus_onyx::client::BaseClient::new();
	///     let file = client.get_file(&std::path::PathBuf::from("path/to/some/image"), None).await;
	/// };
	/// ```
	pub async fn get_file(
		&self,
		_path: &std::path::Path,
		_max_age: Option<std::time::Duration>,
	) -> Box<dyn std::future::Future<Output = Result<(String, Vec<u8>), Box<dyn std::any::Any>>>> {
		todo!()
	}

	/// Retrieve full URL of a document. Useful for example for sharing the public URL of an item in the `/public` folder.
	///
	/// # Arguments
	///
	/// - path : path relative to the module root.
	///
	/// # Returns
	///
	/// The full URL of the item, including the storage origin
	pub fn get_item_url(&self, _path: &std::path::Path) -> &str {
		todo!()
	}

	/// Store object at given path. Triggers synchronization.
	///
	/// See `declareType()`
	///
	/// # Arguments
	///
	/// - type_alias : unique type of this object within this module.
	/// - path : path relative to the module root.
	/// - object : a serializable object to be stored at the given path. Must be serializable as JSON.
	///
	/// # Example
	///
	/// ```
	/// #[derive(serde::Serialize)]
	/// struct Bookmark{
	///     url: String,
	///     description: String,
	///     tags: Vec<String>,
	/// }
	///
	/// let bookmark = Bookmark{
	///     url: String::from("http://unhosted.org"),
	///     description: String::from("Unhosted Adventures"),
	///     tags: vec![
	///         String::from("unhosted"),
	///         String::from("remotestorage"),
	///         String::from("no-backend")
	///     ],
	/// };
	///
	/// let path = std::path::PathBuf::from("/bookmark.json");
	///
	/// let mut client = pontus_onyx::client::BaseClient::new();
	/// async {
	///     client.store_object("bookmark", &path, bookmark).await;
	/// };
	/// ```
	pub async fn store_object<Object>(
		&mut self,
		_type_alias: &str,
		_path: &std::path::Path,
		_object: Object,
	) -> Box<dyn std::future::Future<Output = Result<String, Box<dyn std::any::Any>>>>
	where
		Object: serde::ser::Serialize,
	{
		todo!()
	}

	/// Store raw data at a given path.
	///
	/// # Arguments
	///
	/// - mime_type : MIME media type of the data being stored
	/// - path : path relative to the module root
	/// - body : raw data to store
	///
	/// # Returns
	///
	/// A promise for the created/updated revision (ETag)
	///
	/// # Example
	///
	/// ```
	/// async {
	///     let mut client = pontus_onyx::client::BaseClient::new();
	///     client.store_file(&pontus_onyx::ContentType::from("text/html"), &std::path::PathBuf::from("index.html"), b"<h1>Hello World!</h1>").await;
	/// };
	/// ````
	pub async fn store_file(
		&mut self,
		_mime_type: &crate::ContentType,
		_path: &std::path::Path,
		_body: &[u8],
	) -> Box<dyn std::future::Future<Output = Result<String, Box<dyn std::any::Any>>>> {
		todo!()
	}

	/// Remove node at given path from storage. Triggers synchronization.
	///
	/// # Arguments
	///
	/// - path : path relative to the module root.
	pub fn remove(
		&mut self,
		_path: &std::path::Path,
	) -> Box<dyn std::future::Future<Output = Result<String, Box<dyn std::any::Any>>>> {
		todo!()
	}

	/// `BaseClient` offers a single event, named change, which you can add a handler for using this function.
	///
	/// Using this event, you can stay informed about data changes, both remote (from other devices or browsers), as well as locally (e.g. other browser tabs).
	///
	/// In order to determine where a change originated from, look at the origin property of the incoming event, in `ChangeEvent`.
	pub fn on_change(&mut self, _listener: Box<dyn FnMut(ChangeEvent)>) {
		todo!()
	}

	/// TODO: document
	pub fn flush(&mut self, _path: &std::path::Path) {
		todo!()
	}

	/// Declare a remoteStorage object type using a JSON schema.
	///
	/// Visit [http://json-schema.org](http://json-schema.org) for details on how to use JSON Schema.
	///
	/// # Arguments
	///
	/// - alias : a type alias/shortname
	/// - uri : JSON-LD URI of the schema. Automatically generated if none given
	/// - schema : a JSON Schema object describing the object type
	pub fn declare_type(&mut self, _alias: &str, _uri: Option<&str>, _schema: serde_json::Value) {
		todo!()
	}

	/// Validate an object against the associated schema.
	///
	/// # Arguments
	///
	/// - object : JS object to validate. Must have a `@context` property.
	pub fn validate(&self, _object: serde_json::Value) -> Result<(), ValidationError> {
		todo!()
	}
}

pub struct ChangeEvent {
	/// Absolute path of the changed node, from the storage root
	pub path: std::path::PathBuf,
	/// Path of the changed node, relative to this baseclient's scope root
	pub relative_path: std::path::PathBuf,
	/// See origin descriptions below
	pub origin: EventOrigin,
	/// Old body of the changed node (local version in conflicts; undefined if creation)
	pub old_value: Option<Vec<u8>>,
	/// New body of the changed node (remote version in conflicts; undefined if deletion)
	pub new_value: Vec<u8>,
	/// Old contentType of the changed node (local version for conflicts; undefined if creation)
	pub old_content_type: Option<crate::ContentType>,
	/// New contentType of the changed node (remote version for conflicts; undefined if deletion)
	pub new_content_type: crate::ContentType,
}

pub enum EventOrigin {
	/// Events with origin window are fired whenever you change a value by calling a method on the BaseClient; these are disabled by default.
	Window,
	/// Events with origin local are fired conveniently during the page load, so that you can fill your views when the page loads.
	Local,
	/// Events with origin remote are fired when remote changes are discovered during sync.
	Remote,
	/// Events with origin conflict are fired when a conflict occurs while pushing out your local changes to the remote store.
	Conflict,
}

pub struct ValidationError {
	pub error: Option<String>,
	pub missing: Vec<String>,
}

pub enum CacheStrategy {
	All,
	Seen,
	Flush,
}
impl Default for CacheStrategy {
	fn default() -> Self {
		Self::All
	}
}
