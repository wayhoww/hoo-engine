#[macro_export]
macro_rules! derivable {
    ($prelude: expr => $conclude: expr) => {
        (!($prelude) || ($conclude))
    };
}

#[macro_export]
macro_rules! check {
    ($cond: expr) => {
        if !($cond) {
            return Err(format!(
                "{}:{} - {}: failed: {}",
                file!(),
                line!(),
                module_path!(),
                stringify!($cond)
            ));
        }
    };
}

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debug_only {
    ($($tokens:tt)*) => {
        $($tokens)*
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debug_only {
    ($($tokens:tt)*) => {};
}

#[macro_export]
macro_rules! rcmut {
    ($e: expr) => {
        std::rc::Rc::new(std::cell::RefCell::new($e))
    };
}

#[macro_export]
macro_rules! hoo_log {
    ($($arg:tt)*) => {{
        &println!($($arg)*);
    }};
}

#[macro_export]
macro_rules! wait_sync {
    ($e: expr) => {
        futures::executor::block_on($e)
    };
}
