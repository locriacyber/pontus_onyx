/*
TODO :
	Clients SHOULD also handle the case where a response takes too long
	to arrive, or where no response is received at all.
*/
/*
TODO :
* <storage_root> example : 'https://example.com:8080/path/to/storage' (host, port and
	path prefix; note there is no trailing slash)
* <access_token> as per [OAUTH]. The token SHOULD be hard to
	guess and SHOULD NOT be reused from one client to another. It
	can however be reused in subsequent interactions with the same
	client, as long as that client is still trusted. Example:
	'ofb24f1ac3973e70j6vts19qr9v2eei'
* <storage_api>, always 'draft-dejong-remotestorage-16' for this
	alternative version of the specification.

client request = <storage_root> with
	'/' plus one or more <folder> '/' strings indicating a path in the
	folder tree, followed by zero or one <document> strings
*/

/*
TODO : disabled functionalities ?
	- (rs.js) modules
	- access
*/

mod base_client;
mod storage;

pub use base_client::*;
pub use storage::*;
