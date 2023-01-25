mod credential;
use credential::*;
mod file;
use file::*;
mod download;
use download::*;
mod upload;
use upload::*;

use crate::FileHost;
use async_trait::async_trait;
use futures::io::AsyncBufRead;

pub struct Zippyshare {
    credential: Credential,
}

pub struct UploadSetting {
    pub private: bool,
}

pub struct DownloadSetting {}

#[async_trait]
impl FileHost for Zippyshare {
    type Credential = Credential;
    type File = File;
    type UploadSetting = UploadSetting;
    type DownloadSetting = DownloadSetting;

    fn new(credntial: Self::Credential) -> Self {
        Zippyshare {
            credential: credntial,
        }
    }

    fn max_file_size(&self) -> usize {
        500 * 1000 * 1000
    }

    async fn download(
        &self,
        file: Self::File,
        _setting: Self::DownloadSetting,
    ) -> anyhow::Result<Box<dyn AsyncBufRead + Send + Sync + Unpin>> {
        download_file(file.get_server_id(), file.get_file_id()).await
    }

    async fn upload<'a>(
        &self,
        name: &'a str,
        reader: Box<dyn AsyncBufRead + Send + Sync + Unpin>,
        len: Option<usize>,
        setting: Self::UploadSetting,
    ) -> anyhow::Result<Self::File> {
        if len.is_some() && len.unwrap() > self.max_file_size() {
            return Err(anyhow::anyhow!(format!(
                "reader is larger than {}",
                self.max_file_size()
            )));
        }
        let uri = upload_file(
            name,
            reader,
            len,
            setting.private,
            self.credential.get_ziphash(),
            self.credential.get_zipname(),
        )
        .await?;
        Self::File::try_from(uri)
    }
}
