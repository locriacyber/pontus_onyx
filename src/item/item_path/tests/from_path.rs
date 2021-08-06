use crate::item::{ItemPath, ItemPathPart};

#[test]
fn jmnk3j1xv8mq7cgvifwpz43() {
	assert_eq!(
		ItemPath::from(std::path::Path::new("a/b/c.txt")).0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Document(String::from("c.txt"))
		]
	);
}

#[test]
fn u2wb0ag5vhk0xhmd460() {
	assert_eq!(
		ItemPath::from(std::path::Path::new("a/b/c/")).0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Folder(String::from("c"))
		]
	);
}

#[test]
fn ap4x45tny4jekferziyr() {
	assert_eq!(
		ItemPath::from(std::path::Path::new("a\\b/c")).0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Document(String::from("c"))
		]
	);
}

#[test]
fn al7alq4uqj6mnnma1g() {
	assert_eq!(
		ItemPath::from(std::path::Path::new("C:\\a\\b")).0,
		vec![
			ItemPathPart::Folder(String::from("C:")),
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Document(String::from("b"))
		]
	);
}

#[test]
fn hr1lrd0v64hrpypoq() {
	assert_eq!(
		ItemPath::from(std::path::Path::new("\\\\SERVER\\a\\b")).0,
		vec![
			ItemPathPart::Folder(String::from("SERVER")),
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Document(String::from("b"))
		]
	);
}

#[test]
fn bjwt3pbcft0gacq284s() {
	assert_eq!(
		ItemPath::from(std::path::Path::new("\\a\\\\b\\c")).0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Document(String::from("c"))
		]
	);
}
