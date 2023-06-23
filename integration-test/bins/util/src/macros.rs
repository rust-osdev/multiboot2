#[macro_export]
macro_rules! println {
    () => {
        $crate::println!("")
    };
    ($($arg:tt)*) => {
        $crate::debugcon::_print(format_args!($($arg)*));
        $crate::debugcon::_print(format_args!("\n"));
    };
}
