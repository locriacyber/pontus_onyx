// TODO : do not load all document content in memory !

impl super::Database {
	pub fn new(
		source: &super::DataSource,
		listener: Option<super::EventListener>,
	) -> Result<Self, ErrorNewDatabase> {
		match source {
			super::DataSource::Memory(item) => match item {
				crate::Item::Folder { .. } => Ok(Self {
					content: item.clone(),
					listener,
				}),
				crate::Item::Document { .. } => Err(ErrorNewDatabase::WorksOnlyForFolder),
			},
			#[cfg(feature = "server_bin")]
			super::DataSource::File(path) => match path_to_item(path, &listener) {
				Ok((_, data)) => Ok(Self {
					content: *data,
					listener,
				}),
				Err(e) => Err(e),
			},
			#[cfg(not(feature = "server_bin"))]
			super::DataSource::File(_) => Err(ErrorNewDatabase::DisabledFeature(String::from(
				"server_bin",
			))),
		}
	}
}

#[cfg(feature = "server_bin")]
pub fn path_to_item(
	path: &std::path::Path,
	listener: &Option<super::EventListener>,
) -> Result<(String, Box<crate::Item>), super::ErrorNewDatabase> {
	if !path.exists() {
		return Err(super::ErrorNewDatabase::FileDoesNotExists);
	}

	if path.is_dir() {
		match std::fs::read_dir(path.to_path_buf()) {
			Ok(items) => {
				let content: Vec<Result<(String, Box<crate::Item>), super::ErrorNewDatabase>> =
					items
						.map(|item| match item {
							Ok(entry) => {
								if entry.path().is_dir() {
									path_to_item(&entry.path(), listener)
								} else if entry.path().is_file() {
									let filename = String::from(
										entry.file_name().as_os_str().to_str().unwrap(),
									);

									let mut podata = path.to_path_buf();
									podata.push(format!(".{}.podata.toml", filename));

									let podata = std::fs::read(podata);

									let podata: super::DataDocument = match podata {
										Ok(podata) => toml::from_slice(&podata).unwrap_or_default(),
										Err(_) => super::DataDocument::default(),
									};

									match std::fs::read(entry.path()) {
										Ok(bytes) => Ok((
											filename,
											Box::new(crate::Item::Document {
												content: bytes,
												content_type: podata.content_type,
												etag: podata.etag,
												last_modified: podata.last_modified,
											}),
										)),
										Err(e) => Err(super::ErrorNewDatabase::IOError(e)),
									}
								} else {
									Err(super::ErrorNewDatabase::WrongSource)
								}
							}
							Err(e) => Err(super::ErrorNewDatabase::IOError(e)),
						})
						.filter(|e| {
							if let Ok((name, _)) = e {
								if name.ends_with(".podata.toml") {
									return false;
								}
							}

							return true;
						})
						.collect();

				if content.iter().all(|e| !e.is_err()) {
					let folder_name = path.file_name().unwrap().to_str().unwrap();

					let mut podata = path.to_path_buf();
					podata.push(".folder.podata.toml");

					let podata = std::fs::read(podata);

					let podata: super::DataFolder = match podata {
						Ok(podata) => toml::from_slice(&podata).unwrap_or_default(),
						Err(_e) => {
							let result = super::DataFolder::default();

							/* TODO :
							if let std::io::ErrorKind::NotFound = e.kind() {
								tx.send(Event::Create{})
							}
							*/

							result
						}
					};

					Ok((
						String::from(folder_name),
						Box::new(crate::Item::Folder {
							etag: podata.etag,
							content: content
								.into_iter()
								.map(|e| e.unwrap())
								.collect::<std::collections::HashMap<crate::ItemPath, Box<crate::Item>>>(
							),
						}),
					))
				} else {
					Err(match content.into_iter().find(|e| e.is_err()) {
						Some(Ok(_)) => panic!("error #SGR-573"),
						Some(Err(e)) => e,
						None => panic!("error #NYH-812"),
					})
				}
			}
			Err(e) => Err(super::ErrorNewDatabase::IOError(e)),
		}
	} else if path.is_file() {
		match std::fs::read(&path) {
			Ok(content) => {
				let content: Result<super::DataMonolyth, bincode::Error> =
					bincode::deserialize(&content);

				match content {
					Ok(monolyth) => Ok((
						String::from(path.file_name().unwrap().to_str().unwrap()),
						Box::new(monolyth.content),
					)),
					Err(e) => Err(super::ErrorNewDatabase::DeserializeError(e)),
				}
			}
			Err(e) => Err(super::ErrorNewDatabase::IOError(e)),
		}
	} else {
		Err(super::ErrorNewDatabase::WrongSource)
	}
}

#[derive(Debug)]
pub enum ErrorNewDatabase {
	DeserializeError(bincode::Error),
	FileDoesNotExists,
	IOError(std::io::Error),
	WorksOnlyForFolder,
	WrongSource,
	DisabledFeature(String),
}
impl std::fmt::Display for ErrorNewDatabase {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		match self {
			Self::DeserializeError(e) => {
				f.write_fmt(format_args!("the format of this database is wrong : {}", e))
			}
			Self::FileDoesNotExists => f.write_str("the specified file does not exists"),
			Self::IOError(e) => f.write_fmt(format_args!(
				"there is an error while reading database file : {}",
				e
			)),
			Self::WorksOnlyForFolder => f.write_str("this item should be only type Item::Folder"),
			Self::WrongSource => f.write_str("this database can not be created from this source"),
			Self::DisabledFeature(feature) => f.write_fmt(format_args!(
				"this feature is disabled, it needs `{}` feature at least",
				feature
			)),
		}
	}
}
