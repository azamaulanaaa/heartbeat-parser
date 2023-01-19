use anyhow::Result;
use regex::Regex;

fn verify_id<'a>(id: &'a str) -> bool {
    let re = Regex::new(r"https://www[\d]+\.zippyshare.com/v/[\w\d]+/file\.html").unwrap();
    return re.is_match(id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_id_test() -> Result<()> {
        struct TestCase<'a> {
            problem: &'a str,
            solution: bool,
        }
        let function = verify_id;

        let testcases = [
            TestCase {
                problem: "https://www114.zippyshare.com/v/UfqlE33b/file.html",
                solution: true,
            },
            TestCase {
                problem: "https://www.zippyshare/v/asdkfj23/file.html",
                solution: false,
            },
        ];

        for testcase in testcases {
            let solution = function(testcase.problem);
            assert_eq!(solution, testcase.solution);
        }
        Ok(())
    }
}
