use crate::utils::{gen_boundary, Multipart, MultipartContentEnum, MultipartField};
use futures::AsyncBufRead;
use regex::Regex;
use surf::http::Method;
use surf::{Client, Request, Url};

pub async fn upload_file<'a>(
    name: &'a str,
    reader: Box<dyn AsyncBufRead + Send + Sync + Unpin>,
    len: Option<usize>,
    private: bool,
    ziphash: &'a str,
    zipname: &'a str,
) -> anyhow::Result<String> {
    let client = Client::new();

    let server_id = {
        let req = {
            let uri = "https://www.zippyshare.com";
            let url = Url::parse(uri)?;
            let req = Request::builder(Method::Post, url);

            req.build()
        };
        let mut res = match client.send(req).await {
            Ok(res) => res,
            Err(e) => return Err(e.into_inner()),
        };

        let problem = match res.body_string().await {
            Ok(v) => v,
            Err(e) => return Err(e.into_inner()),
        };

        get_server_id(problem.as_str())?
    };

    let problem = {
        let req = {
            let url = {
                let uri = format!("https://www{}.zippyshare.com/upload", server_id);
                Url::parse(uri.as_str())?
            };
            let boundary = gen_boundary(
                16,
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-",
            )?;
            let content_type = format!("multipart/form-data; boundary={}", boundary);
            let body = {
                let name_field = MultipartField {
                    name: "name",
                    data: MultipartContentEnum::Str(name),
                };
                let reader_field = MultipartField {
                    name: "file",
                    data: MultipartContentEnum::Reader(reader, name, len),
                };
                let private_field = if private {
                    MultipartField {
                        name: "private",
                        data: MultipartContentEnum::Str("true"),
                    }
                } else {
                    MultipartField {
                        name: "notprivate",
                        data: MultipartContentEnum::Str("true"),
                    }
                };
                let ziphash_field = MultipartField {
                    name: "ziphash",
                    data: MultipartContentEnum::Str(ziphash),
                };
                let zipname_field = MultipartField {
                    name: "zipname",
                    data: MultipartContentEnum::Str(zipname),
                };

                Multipart::new(name_field)
                    .chain(reader_field)
                    .chain(private_field)
                    .chain(ziphash_field)
                    .chain(zipname_field)
                    .into_body(boundary.as_str())
            };
            let req = Request::builder(Method::Post, url)
                .header("Content-Type", content_type)
                .body(body)
                .build();
            req
        };

        let mut res = match client.send(req).await {
            Ok(res) => res,
            Err(e) => return Err(e.into_inner()),
        };

        let data = match res.body_string().await {
            Ok(v) => v,
            Err(e) => return Err(e.into_inner()),
        };

        data
    };

    let uri = get_file_uri(problem.as_str())?;

    Ok(uri)
}

fn get_server_id<'a>(problem: &'a str) -> anyhow::Result<String> {
    let re = Regex::new(r"server\s=\s'www([\d]+)';").unwrap();
    let cap = match re.captures(problem) {
        Some(val) => val,
        None => return Err(anyhow::anyhow!("unable to recognize the problem")),
    };
    let server_id = String::from(&cap[1]);

    Ok(server_id)
}

fn get_file_uri<'a>(problem: &'a str) -> anyhow::Result<String> {
    let re = Regex::new(r"\[url=(https://www[\d]+.zippyshare.com/v/[\w\d]+/file\.html)\]").unwrap();
    let cap = match re.captures(problem) {
        Some(val) => val,
        None => return Err(anyhow::anyhow!("unable to recognize the problem")),
    };
    let uri = String::from(&cap[1]);

    Ok(uri)
}

#[cfg(test)]
mod tests {
    use async_std::io::Cursor;

    #[tokio::test]
    async fn upload_file() -> anyhow::Result<()> {
        struct TestCase<'a> {
            name: &'a str,
            reader: Box<dyn super::AsyncBufRead + Send + Sync + Unpin>,
            len: Option<usize>,
            private: bool,
            ziphash: &'a str,
            zipname: &'a str,
        }

        let testcases = [TestCase {
            name: "name.txt",
            reader: Box::new(Cursor::new("abcd")),
            len: Some(4),
            private: false,
            ziphash: "",
            zipname: "",
        }];

        for testcase in testcases {
            let file_uri = super::upload_file(
                testcase.name,
                testcase.reader,
                testcase.len,
                testcase.private,
                testcase.ziphash,
                testcase.zipname,
            )
            .await?;
            assert_ne!(file_uri, String::from(""));
        }

        Ok(())
    }

    #[test]
    fn get_server_id() -> anyhow::Result<()> {
        struct TestCase<'a> {
            problem: &'a str,
            solution: String,
        }
        let function = super::get_server_id;

        let testcases = [
            TestCase {
                problem: "<script type=\"text/javascript\">\nvar uploadId = 'HZ2E4A1F84CDBA4AFA9C09E33CFA0CADB7';\nvar server = 'www53';\n</script>",
                solution: String::from("53"),
            },
        ];

        for testcase in testcases {
            let solution = function(testcase.problem);
            assert_eq!(solution?, testcase.solution);
        }

        Ok(())
    }

    #[test]
    fn get_file_uri() -> anyhow::Result<()> {
        struct TestCase<'a> {
            problem: &'a str,
            solution: &'a str,
        }
        let function = super::get_file_uri;

        let testcases = [
            TestCase {
                problem: "<input type=\"text\" style=\"width: 550px;\" onclick=\"this.select();\" value=\"[url=https://www53.zippyshare.com/v/GbGVeLvy/file.html][img=//www53.zippyshare.com/scaled/GbGVeLvy/file.html][/img][/url]\" class=\"text_field\"/>",
                solution: "https://www53.zippyshare.com/v/GbGVeLvy/file.html",
            },
        ];

        for testcase in testcases {
            let solution = function(testcase.problem)?;
            assert_eq!(testcase.solution, solution.as_str());
        }

        Ok(())
    }
}
