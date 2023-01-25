use anyhow::{anyhow, Result};
use http_types::cookies::{Cookie, CookieJar};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use surf::http::{Method, Url};
use surf::{Client, Request, Response};

#[derive(Clone)]
pub struct AuthToken {
    pub ziphash: String,
    pub zipname: String,
}

impl AuthToken {
    pub fn empty() -> AuthToken {
        AuthToken {
            ziphash: String::from(""),
            zipname: String::from(""),
        }
    }

    pub async fn authenticate(credential: Credential<'_>) -> Result<AuthToken> {
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
                match req.body_form(&credential) {
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

        return Ok(AuthToken {
            ziphash: cookiejar.get("ziphash").unwrap().value().to_string(),
            zipname: cookiejar.get("zipname").unwrap().value().to_string(),
        });
    }
}

pub struct Credential<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

impl Serialize for Credential<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Credential", 2)?;
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
            credential: super::Credential<'a>,
            status: bool,
        }

        let testcases = [
            TestCase {
                credential: super::Credential {
                    username: "amhdevil",
                    password: "devil1234",
                },
                status: true,
            },
            TestCase {
                credential: super::Credential {
                    username: "amhdevil",
                    password: "devil12345",
                },
                status: false,
            },
        ];

        for testcase in testcases {
            let auth_token = super::AuthToken::authenticate(testcase.credential).await;
            if testcase.status {
                let auth_token = auth_token?;
                assert!(!auth_token.ziphash.is_empty());
                assert!(!auth_token.zipname.is_empty());
            } else {
                assert!(auth_token.is_err());
            }
        }

        Ok(())
    }
}
