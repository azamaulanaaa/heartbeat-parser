use anyhow::{anyhow, Result};
use http_types::cookies::{Cookie, CookieJar};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use surf::http::{Method, Url};
use surf::{Client, Request, Response};

#[derive(Clone)]
pub struct Credential {
    ziphash: String,
    zipname: String,
}

impl Credential {
    pub fn empty() -> Credential {
        Credential {
            ziphash: String::from(""),
            zipname: String::from(""),
        }
    }

    pub async fn new(form: CredentialForm<'_>) -> Result<Credential> {
        let client = Client::new();
        let mut cookiejar = CookieJar::new();

        {
            let req = {
                let uri = "https://www.zippyshare.com";
                let url = Url::parse(uri)?;
                Request::builder(Method::Post, url).build()
            };

            let res: Response = match client.send(req).await {
                Ok(res) => res,
                Err(e) => return Err(e.into_inner()),
            };

            for cookie_string in res.header("Set-Cookie").iter() {
                for cookie_string in cookie_string.iter() {
                    let cookie = match Cookie::parse(cookie_string.to_string()) {
                        Ok(cookie) => cookie,
                        Err(_e) => continue,
                    };

                    cookiejar.add(cookie);
                }
            }
        }

        {
            let req = {
                let uri = "https://www.zippyshare.com/services/login";
                let url = Url::parse(uri)?;
                let cookie_string: String = cookiejar
                    .iter()
                    .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
                    .collect::<Vec<String>>()
                    .join("; ");
                let mut req = Request::builder(Method::Post, url)
                    .header("Cookie", cookie_string)
                    .build();
                match req.body_form(&form) {
                    Ok(_) => {}
                    Err(e) => return Err(e.into_inner()),
                }
                req
            };

            let res: Response = match client.send(req).await {
                Ok(res) => res,
                Err(e) => return Err(e.into_inner()),
            };

            for cookie_string in res.header("Set-Cookie").iter() {
                for cookie_string in cookie_string.iter() {
                    let cookie = match Cookie::parse(cookie_string.to_string()) {
                        Ok(cookie) => cookie,
                        Err(_e) => continue,
                    };

                    cookiejar.add(cookie);
                }
            }
        }

        {
            if cookiejar.get("zipname").is_none() || cookiejar.get("ziphash").is_none() {
                return Err(anyhow!("username or password do not match."));
            }
        }

        return Ok(Credential {
            ziphash: cookiejar.get("ziphash").unwrap().value().to_string(),
            zipname: cookiejar.get("zipname").unwrap().value().to_string(),
        });
    }

    pub fn get_ziphash<'a>(&'a self) -> &'a str {
        self.ziphash.as_str()
    }

    pub fn get_zipname<'a>(&'a self) -> &'a str {
        self.zipname.as_str()
    }
}

pub struct CredentialForm<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

impl Serialize for CredentialForm<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("CredentialForm", 2)?;
        s.serialize_field("login", &self.username)?;
        s.serialize_field("pass", &self.password)?;
        s.end()
    }
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn credential_new() -> anyhow::Result<()> {
        struct TestCase<'a> {
            form: super::CredentialForm<'a>,
            status: bool,
        }

        let testcases = [
            TestCase {
                form: super::CredentialForm {
                    username: "amhdevil",
                    password: "devil1234",
                },
                status: true,
            },
            TestCase {
                form: super::CredentialForm {
                    username: "amhdevil",
                    password: "devil12345",
                },
                status: false,
            },
        ];

        for testcase in testcases {
            let credential = super::Credential::new(testcase.form).await;
            if testcase.status {
                let credential = credential?;
                assert!(!credential.ziphash.is_empty());
                assert!(!credential.zipname.is_empty());
            } else {
                assert!(credential.is_err());
            }
        }

        Ok(())
    }
}
