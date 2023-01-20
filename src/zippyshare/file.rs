use anyhow::{anyhow, Error, Result};
use regex::Regex;

#[derive(Debug, PartialEq, Clone)]
pub struct File {
    pub server_id: String,
    pub file_id: String,
}

impl TryFrom<&str> for File {
    type Error = Error;

    fn try_from(src: &str) -> Result<Self> {
        let re = Regex::new(r"https://www([\d]+)\.zippyshare.com/v/([\w\d]+)/file\.html").unwrap();
        let cap = match re.captures(src) {
            Some(val) => val,
            None => return Err(anyhow!("unable to recognize the id.")),
        };

        return Ok(File {
            server_id: String::from(&cap[1]),
            file_id: String::from(&cap[2]),
        });
    }
}

impl TryFrom<String> for File {
    type Error = Error;

    fn try_from(src: String) -> Result<Self> {
        let id = File::try_from(src.as_str())?;
        return Ok(id);
    }
}

impl Into<String> for File {
    fn into(self) -> String {
        let uri = format!(
            "https://www{}.zippyshare.com/v/{}/file.html",
            self.server_id, self.file_id
        );
        return uri;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_tryfrom_str() -> Result<()> {
        struct TestCase<'a> {
            src: &'a str,
            file: File,
        }

        let testcases = [TestCase {
            src: "https://www114.zippyshare.com/v/UfqlE33b/file.html",
            file: File {
                server_id: String::from("114"),
                file_id: String::from("UfqlE33b"),
            },
        }];

        for testcase in testcases {
            let result = File::try_from(testcase.src)?;
            assert_eq!(result, testcase.file);
        }

        Ok(())
    }

    #[test]
    fn file_tryfrom_string() -> Result<()> {
        struct TestCase {
            src: String,
            file: File,
        }

        let testcases = [TestCase {
            src: String::from("https://www114.zippyshare.com/v/UfqlE33b/file.html"),
            file: File {
                server_id: String::from("114"),
                file_id: String::from("UfqlE33b"),
            },
        }];

        for testcase in testcases {
            let file = File::try_from(testcase.src)?;
            assert_eq!(file, testcase.file);
        }
        Ok(())
    }

    #[test]
    fn file_into_string() -> Result<()> {
        struct TestCase {
            file: File,
            result: String,
        }

        let testcases = [TestCase {
            file: File {
                server_id: String::from("114"),
                file_id: String::from("UfqlE33b"),
            },
            result: String::from("https://www114.zippyshare.com/v/UfqlE33b/file.html"),
        }];

        for testcase in testcases {
            let result: String = testcase.file.into();
            assert_eq!(result, testcase.result);
        }
        Ok(())
    }
}
