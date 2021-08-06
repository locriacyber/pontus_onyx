pub mod from_path;
pub mod from_str;

use crate::item::{ItemPath, ItemPathPart};

#[test]
fn pfuh8x4mntyi3ej() {
	let input = ItemPathPart::Document(String::from("gq7tMib.itemdata.json"));
	assert_eq!(input.check_validity(false), Ok(()));
}

#[test]
fn b2auwz1qizhfkrolm() {
	let input = ItemPathPart::Document(String::new());
	assert_eq!(
		input.check_validity(false),
		Err(String::from("should not be empty"))
	);
}

#[test]
fn hf1atgq7tibjv22p2whyhrl() {
	let input = ItemPathPart::Document(String::from("gq7t\0Mib.itemdata.json"));
	assert_eq!(
		input.check_validity(false),
		Err(format!("`{}` should not contains `\\0` character", input))
	);
}

#[test]
fn ptv8v25u24o0u4q3() {
	let input = ItemPathPart::Document(String::from("gq7tMib.itemdata.json"));
	assert_eq!(
		input.check_validity(true),
		Err(format!(
			"`{}` should not contains `.itemdata.` string",
			input
		))
	);
}

//////////////////////////////////////////////////////////////////////////

#[test]
fn pqxxd8pzob0a8mk182hn() {
	assert_eq!(ItemPath::from("A/").parent(), Some(ItemPath::from("")));
}

#[test]
fn vuzxh45545c6pdbh7azm() {
	assert_eq!(ItemPath::from("").parent(), None);
}

//////////////////////////////////////////////////////////////////////////

#[test]
fn xes8rxrql76hb() {
	assert_eq!(
		ItemPath::from("")
			.joined(&crate::item::ItemPathPart::Folder(String::new()))
			.unwrap(),
		ItemPath::from("")
	);
}

#[test]
fn bp6kvtpdcyhu5ip8() {
	assert_eq!(
		ItemPath::from("")
			.joined(&crate::item::ItemPathPart::Folder(String::from("A")))
			.unwrap(),
		ItemPath::from("A/")
	);
}

#[test]
fn h8br2stuj50joa() {
	assert_eq!(
		ItemPath::from("")
			.joined(&crate::item::ItemPathPart::Folder(String::from("A")))
			.unwrap()
			.joined(&crate::item::ItemPathPart::Folder(String::new()))
			.unwrap()
			.joined(&crate::item::ItemPathPart::Folder(String::from("AA")))
			.unwrap(),
		ItemPath::from("A/AA/")
	);
}

//////////////////////////////////////////////////////////////////////////

#[test]
fn kowoqexgrbp() {
	assert_eq!(ItemPath::from("").ancestors(), vec![ItemPath::from(""),]);
}

#[test]
fn hf40iqi11jtqn6hhqrxttbgj() {
	assert_eq!(ItemPath::from("/").ancestors(), vec![ItemPath::from(""),]);
}
