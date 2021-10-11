#[macro_export]
macro_rules! method {
    ( GET ) => {
        reqwest::Client::get
    };
    ( POST ) => {
        reqwest::Client::post
    };
}

#[macro_export]
macro_rules! assert_api {
    (
        $method:ident $url:literal,
        $input:expr => $output:pat $(,)?
    ) => {
        let body = {
            let client = reqwest::Client::new();
            $crate::method!($method)(&client, dbg!(format!("http://127.0.0.1:8080{}", $url)))
                .json(&$input)
                .send()
                .await
                .expect("Failed to perform HTTP request")
                .json()
                .await
                .expect("Failed to convert request output")
        };
        std::assert_matches::assert_matches!(body, $output);
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
