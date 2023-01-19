use anyhow::{anyhow, Error, Result};
use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct ID {
    server_id: String,
    file_id: String,
}

impl TryFrom<&str> for ID {
    type Error = Error;

    fn try_from(src: &str) -> Result<Self> {
        let re = Regex::new(r"https://www([\d]+)\.zippyshare.com/v/([\w\d]+)/file\.html").unwrap();
        let cap = match re.captures(src) {
            Some(val) => val,
            None => return Err(anyhow!("unable to recognize the id.")),
        };

        return Ok(ID {
            server_id: String::from(&cap[1]),
            file_id: String::from(&cap[2]),
        });
    }
}

impl TryFrom<String> for ID {
    type Error = Error;

    fn try_from(src: String) -> Result<Self> {
        let id = ID::try_from(src.as_str())?;
        return Ok(id);
    }
}

impl Into<String> for ID {
    fn into(self) -> String {
        let uri = format!(
            "https://www{}.zippyshare.com/v/{}/file.html",
            self.server_id, self.file_id
        );
        return uri;
    }
}

fn get_filename<'a>(problem: &'a str) -> Result<String> {
    let re = Regex::new(r#""/([\w\d_\-\.]+)""#).unwrap();
    let cap = match re.captures(problem) {
        Some(val) => val,
        None => return Err(anyhow!("unable to recognize the problem.")),
    };
    Ok(String::from(&cap[1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_tryfrom_str() -> Result<()> {
        struct TestCase<'a> {
            src: &'a str,
            result: ID,
        }

        let testcases = [TestCase {
            src: "https://www114.zippyshare.com/v/UfqlE33b/file.html",
            result: ID {
                server_id: String::from("114"),
                file_id: String::from("UfqlE33b"),
            },
        }];

        for testcase in testcases {
            let result = ID::try_from(testcase.src)?;
            assert_eq!(result, testcase.result);
        }
        Ok(())
    }

    #[test]
    fn id_tryfrom_string() -> Result<()> {
        struct TestCase {
            src: String,
            result: ID,
        }

        let testcases = [TestCase {
            src: String::from("https://www114.zippyshare.com/v/UfqlE33b/file.html"),
            result: ID {
                server_id: String::from("114"),
                file_id: String::from("UfqlE33b"),
            },
        }];

        for testcase in testcases {
            let result = ID::try_from(testcase.src)?;
            assert_eq!(result, testcase.result);
        }
        Ok(())
    }
    #[test]
    fn id_into_string() -> Result<()> {
        struct TestCase {
            id: ID,
            result: String,
        }

        let testcases = [TestCase {
            id: ID {
                server_id: String::from("114"),
                file_id: String::from("UfqlE33b"),
            },
            result: String::from("https://www114.zippyshare.com/v/UfqlE33b/file.html"),
        }];

        for testcase in testcases {
            let result: String = testcase.id.into();
            assert_eq!(result, testcase.result);
        }
        Ok(())
    }

    #[test]
    fn get_filename_test() -> Result<()> {
        struct TestCase<'a> {
            problem: &'a str,
            solution: String,
        }
        let function = get_filename;

        let testcases = [
            TestCase {
                problem: "\n\n\n<script type=\"text/javascript\">\n    document.getElementById('dlbutton').href = \"/d/UfqlE33b/\" + (690628 % 51245 + 690628 % 913) + \"/Screenshot_20230113_040647.png\";\n    if (document.getElementById('fimage')) {\n        document.getElementById('fimage').href = \"/i/UfqlE33b/\" + (690628 % 51245 + 690628 % 913) + \"/Screenshot_20230113_040647.png\";\n    }\n</script>",
                solution: String::from("Screenshot_20230113_040647.png"),
            },
        ];

        for testcase in testcases {
            let solution = function(testcase.problem);
            assert_eq!(solution?, testcase.solution);
        }
        Ok(())
    }
}
