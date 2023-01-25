use futures::{AsyncBufRead, AsyncReadExt};
use regex::Regex;
use surf::http::Method;
use surf::{Client, Request, Url};

pub async fn download_file<'a>(
    server_id: &'a str,
    file_id: &'a str,
) -> anyhow::Result<Box<dyn AsyncBufRead + Send + Sync + Unpin>> {
    let client = Client::new();

    let (download_id, filename) = {
        let req = {
            let url = {
                let uri = format!(
                    "https://www{}.zippyshare.com/v/{}/file.html",
                    server_id, file_id
                );
                let url = Url::parse(uri.as_str())?;
                url
            };
            Request::builder(Method::Get, url).build()
        };

        let mut res = match client.send(req).await {
            Ok(res) => res,
            Err(e) => return Err(e.into_inner()),
        };

        let mut problem: String = String::from("");
        res.read_to_string(&mut problem).await?;

        let download_id = get_download_id(problem.as_str())?;
        let filename = get_filename(problem.as_str())?;

        (download_id, filename)
    };

    let download_reader = {
        let req = {
            let uri = format!(
                "https://www{}.zippyshare.com/d/{}/{}/{}",
                server_id, file_id, download_id, filename
            );
            let url = Url::parse(uri.as_str())?;
            Request::builder(Method::Get, url).build()
        };

        let mut res = match client.send(req).await {
            Ok(res) => res,
            Err(e) => return Err(e.into_inner()),
        };

        res.take_body().into_reader()
    };

    Ok(download_reader)
}

fn get_filename<'a>(problem: &'a str) -> anyhow::Result<String> {
    let re = Regex::new(r#""/([\w\d_\-\.]+)""#).unwrap();
    let cap = match re.captures(problem) {
        Some(val) => val,
        None => return Err(anyhow::anyhow!("unable to recognize the problem.")),
    };

    let filename = String::from(&cap[1]);
    Ok(filename)
}

fn get_download_id<'a>(problem: &'a str) -> anyhow::Result<String> {
    let re = Regex::new(r"\(([\d]+)\s%\s([\d]+)\s\+\s[\d]+\s%\s([\d]+)\)").unwrap();
    let cap = match re.captures(problem) {
        Some(val) => val,
        None => return Err(anyhow::anyhow!("unable to recognize the problem.")),
    };
    let id = {
        let a = i32::from_str_radix(&cap[1], 10).unwrap();
        let b = i32::from_str_radix(&cap[2], 10).unwrap();
        let c = i32::from_str_radix(&cap[3], 10).unwrap();
        a % b + a % c
    };

    Ok(id.to_string())
}

#[cfg(test)]
mod tests {
    use futures::AsyncReadExt;
    use sha256::digest;

    #[tokio::test]
    async fn download_file_test() -> anyhow::Result<()> {
        struct TestCase<'a> {
            server_id: &'a str,
            file_id: &'a str,
            sha256: &'a str,
        }

        let testcases = [TestCase {
            server_id: "114",
            file_id: "UfqlE33b",
            sha256: "5fcafaf6c1e7fa70d28e35a68939d55c396f7058e34c94d423a1bebdd6f4b678",
        }];

        for testcase in testcases {
            let mut buff = super::download_file(testcase.server_id, testcase.file_id).await?;
            let mut data: Vec<u8> = vec![];
            buff.read_to_end(&mut data).await?;
            let sha256 = digest(data.as_slice());
            assert_eq!(sha256, testcase.sha256)
        }

        Ok(())
    }

    #[test]
    fn get_filename_test() -> anyhow::Result<()> {
        struct TestCase<'a> {
            problem: &'a str,
            solution: String,
        }
        let function = super::get_filename;

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

    #[test]
    fn get_download_id_test() -> anyhow::Result<()> {
        struct TestCase<'a> {
            problem: &'a str,
            solution: String,
        }
        let function = super::get_download_id;

        let testcases = [
            TestCase {
                problem: "\n\n\n<script type=\"text/javascript\">\n    document.getElementById('dlbutton').href = \"/d/UfqlE33b/\" + (690628 % 51245 + 690628 % 913) + \"/Screenshot_20230113_040647.png\";\n    if (document.getElementById('fimage')) {\n        document.getElementById('fimage').href = \"/i/UfqlE33b/\" + (690628 % 51245 + 690628 % 913) + \"/Screenshot_20230113_040647.png\";\n    }\n</script>",
                solution: String::from("24843"),
            },
        ];

        for testcase in testcases {
            let solution = function(testcase.problem);
            assert_eq!(solution?, testcase.solution);
        }
        Ok(())
    }
}
