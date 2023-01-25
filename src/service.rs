use async_trait::async_trait;
use futures::AsyncBufRead;

pub trait Service {
    type AuthToken;
    type File;
}

#[async_trait]
pub trait Download: Service {
    type DownloadSetting;

    async fn download<'a>(
        file: Self::File,
        setting: Self::DownloadSetting,
        auth_token: &'a Self::AuthToken,
    ) -> anyhow::Result<Box<dyn AsyncBufRead + Send + Sync + Unpin>>;
}

#[async_trait]
pub trait Upload: Service {
    type UploadSetting;

    fn max_file_size<'a>(auth_token: &'a Self::AuthToken) -> usize;
    async fn upload<'a>(
        name: &'a str,
        reader: Box<dyn AsyncBufRead + Send + Sync + Unpin>,
        len: Option<usize>,
        setting: Self::UploadSetting,
        auth_token: &'a Self::AuthToken,
    ) -> anyhow::Result<Self::File>;
}
