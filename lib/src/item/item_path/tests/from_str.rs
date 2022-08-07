use crate::item::{ItemPath, ItemPathPart};

#[test]
fn gu6qe5xy4zdl() {
	assert_eq!(
		ItemPath::from("a/b/c").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Document(String::from("c"))
		]
	);
}
#[test]
fn ojni817xbdfsv4lryfol3() {
	assert_eq!(
		ItemPath::from("a/b/c/").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Folder(String::from("c"))
		]
	);
}
#[test]
fn h60ujth6dopz1fbcg() {
	assert_eq!(
		ItemPath::from("a/b/../c/").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("c"))
		]
	);
}
#[test]
fn pwjqrwivvf4es31() {
	assert_eq!(
		ItemPath::from("..").0,
		vec![ItemPathPart::Folder(String::from("")),]
	);
}
#[test]
fn ccyet1ejsei14hxrmberswip() {
	assert_eq!(
		ItemPath::from("a\\b\\c").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Document(String::from("c"))
		]
	);
}
#[test]
fn tuxcgtsmdowij() {
	assert_eq!(
		ItemPath::from("a\\b\\c\\").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Folder(String::from("c"))
		]
	);
}
#[test]
fn jcavyevvgnm60mdlg2g12() {
	assert_eq!(
		ItemPath::from("a\\b\\..\\c\\").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("c"))
		]
	);
}
#[test]
fn v201kvbp5rkamp1m2u62gkd1() {
	assert_eq!(
		ItemPath::from("a\\b/c").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Folder(String::from("b")),
			ItemPathPart::Document(String::from("c"))
		]
	);
}
#[test]
fn wprify4w2e82oaalbbxvjwi() {
	assert_eq!(
		ItemPath::from("").0,
		vec![ItemPathPart::Folder(String::from("")),]
	);
}
#[test]
fn ci40aqtbosaxg50cpl5z() {
	assert_eq!(
		ItemPath::from("/").0,
		vec![ItemPathPart::Folder(String::from("")),]
	);
}
#[test]
fn edtq0renvugb08s03j186ghn() {
	assert_eq!(
		ItemPath::from("/a/").0,
		vec![ItemPathPart::Folder(String::from("a")),]
	);
}

#[test]
fn p3ubrdxjoepapkt1h() {
	assert_eq!(
		ItemPath::from("./a/b").0,
		vec![
			ItemPathPart::Folder(String::from("a")),
			ItemPathPart::Document(String::from("b"))
		]
	);
}

#[test]
fn qm2ek4irkfzrbwriz56() {
	assert_eq!(
		ItemPath::from("a/aa/aaa.txt").ancestors(),
		vec![
			ItemPath::from(""),
			ItemPath::from("a/"),
			ItemPath::from("a/aa/"),
			ItemPath::from("a/aa/aaa.txt"),
		]
	);
}

#[test]
fn vca2gwyljdba7r4xrv8hc386() {
	assert_eq!(
		ItemPath::from("a/aa/").ancestors(),
		vec![
			ItemPath::from(""),
			ItemPath::from("a/"),
			ItemPath::from("a/aa/"),
		]
	);
}
