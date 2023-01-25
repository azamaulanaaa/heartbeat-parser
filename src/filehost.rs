use async_trait::async_trait;
use futures::AsyncBufRead;

#[async_trait]
pub trait FileHost {
    type Credential;
    type File;
    type UploadSetting;
    type DownloadSetting;

    fn new(credntial: Self::Credential) -> Self;
    fn max_file_size(&self) -> usize;
    async fn download(
        &self,
        file: Self::File,
        setting: Self::DownloadSetting,
    ) -> anyhow::Result<Box<dyn AsyncBufRead + Send + Sync + Unpin>>;
    async fn upload<'a>(
        &self,
        name: &'a str,
        reader: Box<dyn AsyncBufRead + Send + Sync + Unpin>,
        len: Option<usize>,
        setting: Self::UploadSetting,
    ) -> anyhow::Result<Self::File>;
}
