use crate::dummies::get_a_hash;
use crate::upload::*;
use std::fs::remove_file;
use std::fs::File;
use std::io::BufRead;
use std::io::Write;

const COOKIES_FILE: &str = ".COOKIES_115.cache";

#[derive(Debug)]
pub struct Runtime {
    session: Option<Session>,
}
impl Runtime {
    pub fn new() -> Self {
        let path = std::env::current_exe();
        if path.is_ok() {
            let mut path = path.unwrap();
            path.pop();
            path.push(COOKIES_FILE);

            if path.exists() {
                let f = File::open(path).expect("should be able to open file");
                if let Ok(session) = serde_json::from_reader(f) {
                    let session = Some(session);
                    return Self { session };
                }
            }
        }
        return Self { session: None };
    }

    pub fn has_cookies(&self) -> bool {
        self.session.is_some()
    }

    pub fn print_cookies(&self) -> String {
        self.session.as_ref().unwrap().cookies.to_owned()
    }

    pub fn set_cookies(&mut self, cookies: &str) -> Result<(), Box<dyn std::error::Error>> {
        let new_sesion = Session::new(cookies.to_owned());
        self.session = Some(new_sesion);

        let mut path = std::env::current_exe()?;
        path.pop();
        path.push(COOKIES_FILE);

        if path.exists() {
            remove_file(&path)?
        }

        let f = File::create(path)?;

        let mut new_session = Session::new(cookies.to_owned());

        new_session.get_key_if_none()?;

        serde_json::to_writer(f, &new_session)?;

        Ok(())
    }

    pub fn clean(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = std::env::current_exe();
        if path.is_ok() {
            let mut path = path.unwrap();
            path.pop();
            path.push(COOKIES_FILE);

            if path.exists() {
                remove_file(path)?;
            }
        }
        Ok(())
    }

    pub fn check_name(&self, name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(ref session) = self.session {
            let folder_id = session.create_folder(0, "TMP_rs115".into())?;

            let hash = get_a_hash();

            let is_uploaded = match session.upload115_sha1(
                name.to_owned(),
                "5".to_owned(),
                hash.to_owned(),
                hash,
                folder_id,
            ) {
                Ok(_) => true,
                Err(e) => {
                    if e.is::<FileNameForbiddenError>() {
                        if session.delete_one(0, folder_id).is_err() {
                            eprintln!("fail to delete the folder TMP_rs115")
                        }
                        return Ok(false);
                    }
                    false
                }
            };
            if session.delete_one(0, folder_id).is_err() {
                eprintln!("fail to delete the folder TMP_rs115")
            }
            if is_uploaded {
                return Ok(true);
            } else {
                return Err(UnknownError().into());
            }
        } else {
            return Err("cookies not set".into());
        }
    }

    pub fn check_name_bulk_to_file<T: BufRead, U: Write>(
        &self,
        file: T,
        mut forbiden_list: Option<U>,
        mut check_fail: Option<U>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref session) = self.session {
            let folder_id = session.create_folder(0, "TMP_rs115".into())?;

            for line in file.lines() {
                let line = line?;
                let hash = get_a_hash();
                match session.upload115_sha1(
                    line.to_owned(),
                    "5".to_owned(),
                    hash.to_owned(),
                    hash,
                    folder_id,
                ) {
                    Ok(_) => {
                        println!("checked {}", line);
                    }
                    Err(e) => {
                        if e.is::<FileNameForbiddenError>() {
                            println!("NAME NOT ALLOW: {}", line);
                            if let Some(ref mut forbiden_list) = forbiden_list {
                                writeln!(forbiden_list, "{}", line)?;
                            }
                        } else {
                            println!("failed to check: {}, cause by: {}", line, e);
                            if let Some(ref mut check_fail) = check_fail {
                                writeln!(check_fail, "{}", line)?;
                            }
                        }
                    }
                };
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }

            if session.delete_one(0, folder_id).is_err() {
                eprintln!("fail to delete the folder TMP_rs115");
            }
            return Ok(());
        } else {
            return Err("cookies not set".into());
        }
    }
}

#[derive(Debug)]
pub(crate) struct UnknownError();
impl std::fmt::Display for UnknownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "check fails")
    }
}
impl std::error::Error for UnknownError {}
