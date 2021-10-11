#[macro_export]
macro_rules! assert_api {
    ( $( $tt:tt )* ) => {
        todo!()
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
