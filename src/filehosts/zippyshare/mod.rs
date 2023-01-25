mod authtoken;
use authtoken::*;
mod file;
use file::*;
mod download;
use download::*;
mod upload;
use upload::*;

use crate::FileHost;
use async_trait::async_trait;
use futures::io::AsyncBufRead;

pub struct Zippyshare {}

pub struct UploadSetting {
    pub private: bool,
}

pub struct DownloadSetting {}

#[async_trait]
impl FileHost for Zippyshare {
    type AuthToken = AuthToken;
    type File = File;
    type UploadSetting = UploadSetting;
    type DownloadSetting = DownloadSetting;

    fn max_file_size<'a>(_auth_token: &'a self::AuthToken) -> usize {
        500 * 1000 * 1000
    }

    async fn download<'a>(
        file: Self::File,
        _setting: Self::DownloadSetting,
        _auth_token: &'a self::AuthToken,
    ) -> anyhow::Result<Box<dyn AsyncBufRead + Send + Sync + Unpin>> {
        download_file(file.get_server_id(), file.get_file_id()).await
    }

    async fn upload<'a>(
        name: &'a str,
        reader: Box<dyn AsyncBufRead + Send + Sync + Unpin>,
        len: Option<usize>,
        setting: Self::UploadSetting,
        auth_token: &'a Self::AuthToken,
    ) -> anyhow::Result<Self::File> {
        let max_file_size = Zippyshare::max_file_size(auth_token);
        if len.is_some() && len.unwrap() > max_file_size {
            return Err(anyhow::anyhow!(format!(
                "reader is larger than {}",
                max_file_size,
            )));
        }
        let uri = upload_file(
            name,
            reader,
            len,
            setting.private,
            auth_token.ziphash.as_str(),
            auth_token.zipname.as_str(),
        )
        .await?;
        Self::File::try_from(uri)
    }
}
