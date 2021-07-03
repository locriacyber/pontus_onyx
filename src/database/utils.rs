// cargo test --all-features --jobs 1  -- database::utils --nocapture

pub fn is_ok(path: &str) -> Result<(), String> {
	if path.trim().is_empty() {
		return Err(String::from("should not be empty"));
	}

	if path.trim() == "." {
		return Err(String::from("`.` is not allowed"));
	}

	if path.trim() == ".." {
		return Err(String::from("`..` is not allowed"));
	}

	if path.contains('\0') {
		return Err(format!("`{}` should not contains \\0 character", path));
	}

	return Ok(());
}

#[test]
fn pfuh8x4mntyi3ej() {
	let input = "gq7tib";
	assert_eq!(is_ok(&input), Ok(()));
}

#[test]
fn b2auwz1qizhfkrolm() {
	let input = "";
	assert_eq!(is_ok(&input), Err(String::from("should not be empty")));
}

#[test]
fn hf1atgq7tibjv22p2whyhrl() {
	let input = "gq7t\0ib";
	assert_eq!(
		is_ok(&input),
		Err(format!("`{}` should not contains \\0 character", input))
	);
}

pub fn get_parent(input: &std::path::Path) -> std::path::PathBuf {
	std::path::PathBuf::from(format!("{}/", input.parent().unwrap().to_str().unwrap()))
}
