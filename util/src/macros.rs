#[macro_export]
macro_rules! strings {
    [$($x:expr),*] => (vec![$($x.to_string()),*]);
}

#[macro_export]
macro_rules! vec_add {
    ($x:expr, $y:expr) => {{
        let mut x = $x;
        x.append(&mut $y);
        x
    }}
}

