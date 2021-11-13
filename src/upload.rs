#![allow(dead_code)]
use crypto::{digest::Digest, sha1::Sha1};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const INFO_URL: &str = r"https://proapi.115.com/app/uploadinfo";
const CREATE_DIR_URL: &str = "https://webapi.115.com/files/add";
const DELETE_URL: &str = "https://webapi.115.com/rb/delete";
const TARGET_PREFIX: &str = r"U_1_";
const APP_VER: &str = r"29.0.0";
const USER_AGENT_PREFIX: &str = r"Mozilla/5.0 115disk/";
const END_STRING: &str = r"000000";

type MayBeError = Result<(), Box<dyn std::error::Error>>;

#[derive(Debug, Serialize, Deserialize)]

pub(crate) struct Session {
    pub(crate) cookies: String,
    pub(crate) user_id: Option<String>, // may need to request very time?
    pub(crate) user_key: Option<String>,

    #[serde(skip)]
    pub(crate) client: Client,
    #[serde(skip)]
    pub(crate) ua: String,
}

impl Session {
    pub(crate) fn new(cookies: String) -> Self {
        let ua: String = format!("{}{}", USER_AGENT_PREFIX, APP_VER);
        let client = reqwest::blocking::Client::builder()
            .user_agent(&ua)
            .build()
            .expect("TLS backend should be able to initailize");

        Self {
            cookies,
            user_id: None,
            user_key: None,
            client,
            ua,
        }
    }

    pub(crate) fn get_key_if_none(&mut self) -> MayBeError {
        if self.user_id.is_none() || self.user_key.is_none() {
            self.get_user_key()?
        }
        Ok(())
    }

    pub(crate) fn get_user_key(&mut self) -> MayBeError {
        let url = INFO_URL.to_string();
        let res = self
            .client
            .get(url)
            .header("User-Agent", &self.ua)
            .header("Cookie", &self.cookies)
            .send()?;

        let res: UserInfo = res.json()?;
        self.user_key = Some(res.userkey);
        self.user_id = Some(res.user_id.to_string());

        Ok(())
    }

    pub(crate) fn upload115_sha1(
        &self,
        filename: String,
        file_size: String,
        total_hash: String,
        block_hash: String,
        cid: u64,
    ) -> MayBeError {
        let pre_id = block_hash;
        let file_id = total_hash.to_uppercase();
        let quick_id = &file_id;
        let target = TARGET_PREFIX.to_owned() + &cid.to_string();
        let user_id = self.user_id.as_ref().ok_or(UploadError::MissingUserid)?;
        let user_key = self.user_key.as_ref().ok_or(UploadError::MissingUserkey)?;

        let hash = sha1(format!("{}{}{}{}{}", user_id, file_id, quick_id, target, "0").as_str());
        let sig_string = user_key.to_owned() + &hash + END_STRING;
        let sig = sha1(&sig_string);

        let param = [
            ("preid", pre_id),
            ("filename", filename),
            ("quickid", quick_id.into()),
            ("user_id", user_id.into()),
            ("app_ver", APP_VER.into()),
            ("filesize", file_size),
            ("userid", user_id.into()),
            ("exif", "".into()),
            ("target", target),
            ("fileid", total_hash.to_uppercase()),
        ];

        let url = format!(
        "https://uplb.115.com/3.0/initupload.php?isp=0&appid=0&appversion={}&format=json&sig={}",
        APP_VER, sig
    );

        let res = self
            .client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", &self.cookies)
            .header("User-Agent", &self.ua)
            .form(&param)
            .send()
            .map_err(|_| UploadError::RequestError)?;

        let res: UploadResponseJson = res.json().expect("json return will always consistent");

        if res.statuscode == 414 {
            Err(FileNameForbiddenError().into())
        } else if res.statuscode != 0 {
            Err(UploadError::Other(res.statusmsg.js_utf8_decode()?).into())
        } else {
            let item = res.other.get("status").unwrap();
            let err = Err(UploadError::Other(
                "Not succ: ".to_owned() + &res.statusmsg.js_utf8_decode()?,
            )
            .into());

            if !item.is_number() {
                return err;
            }
            let status = item.as_i64().expect("must be number");
            if status != 2 {
                err
            } else {
                Ok(())
            }
        }
    }

    pub(crate) fn create_folder(
        &self,
        pid: u64,
        name: String,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let param = [("pid", pid.to_string()), ("cname", name)];
        let url = CREATE_DIR_URL.to_owned();
        let res: CreateDirResponseJson = self
            .client
            .post(url)
            .form(&param)
            .header("User-Agent", &self.ua)
            .header("Cookie", &self.cookies)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()?
            .json()?;

        if !res.state {
            if let StringOri32::Number(20004) = res.errno {
                // dir exist in the cloud
                Err(UploadError::DirExist.into())
            } else {
                Err("create dir failed...".into())
            }
        } else {
            Ok(res.cid.expect("should return valid cid").parse()?)
        }
    }

    pub(crate) fn delete_one(&self, pid: u64, target: u64) -> MayBeError {
        let param = [("pid", pid), ("fid[0]", target), ("ignore_warn", 1)];

        let url = DELETE_URL.to_owned();
        let res: DeleteResponseJson = self
            .client
            .post(url)
            .form(&param)
            .header("User-Agent", &self.ua)
            .header("Cookie", &self.cookies)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()?
            .json()?;

        if res.state {
            Ok(())
        } else {
            Err(UploadError::DeleteFails.into())
        }
    }

    pub(crate) fn delete_bulk(&self, pid: u64, target_list: Vec<u64>) -> MayBeError {
        let mut param = [("pid".to_owned(), pid), ("ignore_warn".to_owned(), 1)].to_vec();

        for (i, item) in target_list.into_iter().enumerate() {
            let key = format!("fid[{}]", i);
            let value = item;
            param.push((key, value));
        }

        let url = DELETE_URL.to_owned();
        let res: DeleteResponseJson = self
            .client
            .post(url)
            .form(&param)
            .header("User-Agent", &self.ua)
            .header("Cookie", &self.cookies)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()?
            .json()?;

        if res.state {
            Ok(())
        } else {
            Err(UploadError::DeleteFails.into())
        }
    }
}

trait JsUnicodeEncoded {
    fn js_utf8_decode(&self) -> Result<String, Box<dyn std::error::Error>>;
}

use unescape::unescape;
impl JsUnicodeEncoded for String {
    fn js_utf8_decode(&self) -> Result<String, Box<dyn std::error::Error>> {
        unescape(&self).ok_or("js_utf8_decode failed".into())
    }
}

type Other = std::collections::BTreeMap<String, Value>;
#[derive(Deserialize)]
pub(crate) struct UserInfo {
    user_id: i32,
    userkey: String,
    #[serde(flatten)]
    other: Other,
}

#[derive(Deserialize, Debug)]
pub(crate) struct UploadResponseJson {
    statuscode: u32,
    statusmsg: String,
    #[serde(flatten)]
    other: Other,
}

#[derive(Deserialize, Debug)]
pub(crate) struct CreateDirResponseJson {
    state: bool,
    errno: StringOri32,
    cid: Option<String>,
    #[serde(flatten)]
    other: Other,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum StringOri32 {
    S(String),
    Number(i32),
}

#[derive(Deserialize, Debug)]
pub(crate) struct DeleteResponseJson {
    state: bool,
    #[serde(flatten)]
    other: Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct File {
    path: String,
    cid: u64,
}

impl File {
    fn test() {}
}

#[derive(Debug)]
pub(crate) struct FileNameForbiddenError();
impl std::fmt::Display for FileNameForbiddenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "name not allowed by 115, filename may contains word in the censor list"
        )
    }
}
impl std::error::Error for FileNameForbiddenError {}

#[derive(Debug)]
pub(crate) enum UploadError {
    MissingUserid,
    MissingUserkey,
    RequestError,
    DeleteFails,
    DirExist,
    Other(String),
}

impl std::fmt::Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UploadError::MissingUserid => write!(f, "missing userid"),
            UploadError::MissingUserkey => write!(f, "missing userkey"),
            UploadError::Other(ctx) => write!(f, "error: {}", ctx),
            UploadError::RequestError => write!(f, "network request error"),
            UploadError::DirExist => write!(f, "create folder failed, dir already exist"),
            UploadError::DeleteFails => write!(f, "delete failed.."),
        }
    }
}

impl std::error::Error for UploadError {}

fn sha1(content: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.input_str(content);
    hasher.result_str()
}
