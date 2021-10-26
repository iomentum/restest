pub trait IntoUrl {
    fn into_url(self) -> String;
}

impl IntoUrl for &'static str {
    fn into_url(self) -> String {
        if self.starts_with('/') {
            self.to_string()
        } else {
            format!("/{}", self)
        }
    }
}

impl IntoUrl for Vec<Box<dyn ToString>> {
    fn into_url(self) -> String {
        let mut buff = String::new();

        for segment in self {
            buff.push('/');
            let segment = segment.to_string();
            buff.push_str(segment.as_str());
        }

        buff
    }
}
