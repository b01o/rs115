use std::str::FromStr;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

type Sha1Link = String;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Sha1JsonFolderEntity {
    dir_name: String,
    files: Vec<Sha1Link>,
    dirs: Vec<Self>,
}

impl FromStr for Sha1JsonFolderEntity {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let file = File::open(s)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader::<_, Sha1JsonFolderEntity>(reader)?)
    }
}

#[derive(Debug)]
struct Parse115SHA1Error();
impl std::error::Error for Parse115SHA1Error {}

impl std::fmt::Display for Parse115SHA1Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid file")
    }
}
