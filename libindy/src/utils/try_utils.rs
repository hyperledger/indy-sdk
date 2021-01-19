macro_rules! try_cb {
    ($e:expr, $cb:ident, $metrics:expr) => (match $e {
        Ok(val) => val,
        Err(err) => return $cb(Err(::std::convert::From::from(err)), $metrics),
    });
}