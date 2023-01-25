use futures::io::AsyncBufRead;
use http_types::Body;
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;

pub enum MultipartContentEnum<'a> {
    String(String),
    Str(&'a str),
    Reader(
        Box<dyn AsyncBufRead + Send + Sync + Unpin>,
        &'a str,
        Option<usize>,
    ),
}

pub struct MultipartField<'a> {
    pub name: &'a str,
    pub data: MultipartContentEnum<'a>,
}

pub struct Multipart<'a> {
    field: MultipartField<'a>,
    next: Option<Box<Multipart<'a>>>,
}

impl<'a> Multipart<'a> {
    pub fn new(field: MultipartField) -> Multipart {
        Multipart { field, next: None }
    }

    pub fn chain(self, next: MultipartField<'a>) -> Multipart<'a> {
        Multipart {
            field: next,
            next: Some(Box::new(self)),
        }
    }

    pub fn into_body(self, boundary: &'a str) -> Body {
        let mut body = Body::empty();

        let mut current = self;
        loop {
            let boundary_body = {
                let boundary_string = format!("--{}\r\n", boundary);
                let boundary_body = Body::from_string(boundary_string);
                boundary_body
            };
            let data_body = match current.field.data {
                MultipartContentEnum::String(v) => {
                    let data_str = format!(
                        "Content-Disposition: form-data; name=\"{}\"\r\n\r\n{}\r\n",
                        current.field.name, v
                    );
                    let data_body = Body::from_string(data_str);
                    data_body
                }
                MultipartContentEnum::Str(v) => {
                    let data_str = format!(
                        "Content-Disposition: form-data; name=\"{}\"\r\n\r\n{}\r\n",
                        current.field.name, v
                    );
                    let data_body = Body::from_string(data_str);
                    data_body
                }
                MultipartContentEnum::Reader(reader, filename, len) => {
                    let header_str = format!(
                        "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n\r\n",
                        current.field.name, filename
                    );
                    let header_body = Body::from_string(header_str);
                    let data_body = Body::from_reader(reader, len);

                    header_body.chain(data_body)
                }
            };

            body = body.chain(boundary_body).chain(data_body);

            if current.next.is_none() {
                let end_boundary = format!("--{}--", boundary);
                let end_boundary_body = Body::from_string(end_boundary);
                body = body.chain(end_boundary_body);
                break;
            } else {
                current = *current.next.unwrap();
            }
        }

        body
    }
}

pub fn gen_boundary<'a>(length: usize, charset: &'a str) -> anyhow::Result<String> {
    if charset.is_empty() {
        return Err(anyhow::anyhow!("charset is empty"));
    }

    let charset: Vec<char> = charset.chars().collect();
    let distribution = Uniform::from(0..charset.len() - 1);
    let mut rng = thread_rng();
    let mut result = String::with_capacity(length);

    for _ in 0..length {
        let chosen = distribution.sample(&mut rng);
        result.push(*charset.get(chosen).unwrap());
    }

    return Ok(result);
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn multipart_into_body() -> anyhow::Result<()> {
        struct TestCase<'a> {
            args: (super::Multipart<'a>, &'a str),
            body_string: &'a str,
        }

        let testcases = [TestCase {
            args: (
                super::Multipart::new(super::MultipartField {
                    name: "name",
                    data: super::MultipartContentEnum::Str("content"),
                }),
                "boundary",
            ),
            body_string: "--boundary\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\ncontent\r\n--boundary--",
        }];

        for testcase in testcases {
            let body_string = match testcase
                .args
                .0
                .into_body(testcase.args.1)
                .into_string()
                .await
            {
                Ok(v) => v,
                Err(e) => return Err(e.into_inner()),
            };
            assert_eq!(testcase.body_string, body_string)
        }

        Ok(())
    }

    #[test]
    fn gen_boundary() -> anyhow::Result<()> {
        struct TestCase<'a> {
            args: (usize, &'a str),
        }

        let testcases = [
            TestCase { args: (4, "1234") },
            TestCase {
                args: (15, "asdfghjlqwertyuo!@#$%^&*"),
            },
        ];

        for testcase in testcases {
            let boundary: String = super::gen_boundary(testcase.args.0, testcase.args.1)?;
            assert_eq!(boundary.len(), testcase.args.0);
            for charr in boundary.chars() {
                assert!(testcase.args.1.contains(charr))
            }
        }
        Ok(())
    }
}
