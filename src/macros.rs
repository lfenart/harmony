macro_rules! api {
    ($e:expr $(,)?) => {
        format!(
            "{}/v{}{}",
            crate::consts::API_URL,
            crate::consts::API_VERSION,
            $e,
        )
    };
    ($f:expr, $($e:expr),+ $(,)?) => {
        format!(
            "{}/v{}{}",
            crate::consts::API_URL,
            crate::consts::API_VERSION,
            format!($f, $($e,)*),
        )
    };
}
