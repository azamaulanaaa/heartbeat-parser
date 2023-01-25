use async_trait::async_trait;
use futures::AsyncBufRead;

#[async_trait]
pub trait FileHost {
    type AuthToken;
    type File;
    type UploadSetting;
    type DownloadSetting;

    fn max_file_size<'a>(auth_token: &'a Self::AuthToken) -> usize;
    async fn download<'a>(
        file: Self::File,
        setting: Self::DownloadSetting,
        auth_token: &'a Self::AuthToken,
    ) -> anyhow::Result<Box<dyn AsyncBufRead + Send + Sync + Unpin>>;
    async fn upload<'a>(
        name: &'a str,
        reader: Box<dyn AsyncBufRead + Send + Sync + Unpin>,
        len: Option<usize>,
        setting: Self::UploadSetting,
        auth_token: &'a Self::AuthToken,
    ) -> anyhow::Result<Self::File>;
}
