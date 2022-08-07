#[test]
fn jlupvpfbk7wbig1at4h() {
	let AAA = crate::item::Item::new_doc(b"test", "text/plain");
	let AA = crate::item::Item::new_folder(vec![("AAA.txt", AAA.clone())]);
	let A = crate::item::Item::new_folder(vec![("AA", AA.clone())]);
	let root = crate::item::Item::new_folder(vec![("A", A.clone())]);

	assert_eq!(
		root.get_child(&crate::item::ItemPath::from("A/AA/AAA.txt")),
		Some(&AAA)
	);
	assert_eq!(
		root.get_child(&crate::item::ItemPath::from("A/AA/")),
		Some(&AA)
	);
	assert_eq!(
		root.get_child(&crate::item::ItemPath::from("A/AA")),
		Some(&AA)
	);
	assert_eq!(root.get_child(&crate::item::ItemPath::from("A/")), Some(&A));
	assert_eq!(root.get_child(&crate::item::ItemPath::from("A")), Some(&A));
	assert_eq!(
		root.get_child(&crate::item::ItemPath::from("")),
		Some(&root)
	);

	assert_eq!(root.get_child(&crate::item::ItemPath::from("B")), None);
	assert_eq!(root.get_child(&crate::item::ItemPath::from("B/")), None);
	assert_eq!(root.get_child(&crate::item::ItemPath::from("B/BB")), None);
	assert_eq!(root.get_child(&crate::item::ItemPath::from("B/BB/")), None);
}

#[test]
fn j5bmhdxhlgkhdk82rjio3ej6() {
	let mut AAA = crate::item::Item::new_doc(b"test", "text/plain");
	let mut AA = crate::item::Item::new_folder(vec![("AAA.txt", AAA.clone())]);
	let mut A = crate::item::Item::new_folder(vec![("AA", AA.clone())]);
	let mut root = crate::item::Item::new_folder(vec![("A", A.clone())]);

	assert_eq!(
		root.get_child_mut(&crate::item::ItemPath::from("A/AA/AAA.txt")),
		Some(&mut AAA)
	);
	assert_eq!(
		root.get_child_mut(&crate::item::ItemPath::from("A/AA/")),
		Some(&mut AA)
	);
	assert_eq!(
		root.get_child_mut(&crate::item::ItemPath::from("A/AA")),
		Some(&mut AA)
	);
	assert_eq!(
		root.get_child_mut(&crate::item::ItemPath::from("A/")),
		Some(&mut A)
	);
	assert_eq!(
		root.get_child_mut(&crate::item::ItemPath::from("A")),
		Some(&mut A)
	);
	assert!(root
		.get_child_mut(&crate::item::ItemPath::from(""))
		.is_some());

	assert_eq!(root.get_child_mut(&crate::item::ItemPath::from("B")), None);
	assert_eq!(root.get_child_mut(&crate::item::ItemPath::from("B/")), None);
	assert_eq!(
		root.get_child_mut(&crate::item::ItemPath::from("B/BB")),
		None
	);
	assert_eq!(
		root.get_child_mut(&crate::item::ItemPath::from("B/BB/")),
		None
	);
}
