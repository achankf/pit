pub fn create_fetch_client() -> reqwest::Client {
    reqwest::Client::builder()
        .build()
        .expect("unable to set up reqwest client")
}
