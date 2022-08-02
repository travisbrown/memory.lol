use egg_mode::Token;

#[async_trait::async_trait]
pub trait AuthDb {
    type Connection;
    type Error: std::error::Error;

    async fn get_github_name(
        connection: &mut Self::Connection,
        id: u64,
    ) -> Result<Option<String>, Self::Error>;

    async fn get_google_email(
        connection: &mut Self::Connection,
        sub: &str,
    ) -> Result<Option<String>, Self::Error>;

    async fn get_google_sub(
        connection: &mut Self::Connection,
        email: &str,
    ) -> Result<Option<String>, Self::Error>;

    async fn get_twitter_name(
        connection: &mut Self::Connection,
        id: u64,
    ) -> Result<Option<String>, Self::Error>;

    async fn put_github_name(
        connection: &mut Self::Connection,
        id: u64,
        value: &str,
    ) -> Result<(), Self::Error>;

    async fn put_google_email(
        connection: &mut Self::Connection,
        sub: &str,
        value: &str,
    ) -> Result<(), Self::Error>;

    async fn put_twitter_name(
        connection: &mut Self::Connection,
        id: u64,
        value: &str,
    ) -> Result<(), Self::Error>;

    async fn lookup_github_token(
        connection: &mut Self::Connection,
        token: &str,
    ) -> Result<Option<(u64, bool)>, Self::Error>;

    async fn lookup_google_token(
        connection: &mut Self::Connection,
        token: &str,
    ) -> Result<Option<(String, String)>, Self::Error>;

    async fn lookup_twitter_token(
        connection: &mut Self::Connection,
        token: &str,
    ) -> Result<Option<u64>, Self::Error>;

    async fn get_twitter_access_token(
        connection: &mut Self::Connection,
        token: &str,
    ) -> Result<Option<Token>, Self::Error>;

    async fn put_github_token(
        connection: &mut Self::Connection,
        token: &str,
        id: u64,
        gist: bool,
    ) -> Result<(), Self::Error>;

    async fn put_google_token(
        connection: &mut Self::Connection,
        token: &str,
        sub: &str,
    ) -> Result<(), Self::Error>;

    async fn put_twitter_token(
        connection: &mut Self::Connection,
        token: &str,
        id: u64,
        consumer_secret: &str,
        access_key: &str,
        access_secret: &str,
    ) -> Result<(), Self::Error>;
}
