#[macro_export]
macro_rules! args {
    ( $( $x:expr),* ) => {
        {
            #[allow(unused_imports)]
            use sqlx::Arguments;

            let mut temp_args = sqlx::any::AnyArguments::default();
            $(
                temp_args.add($x);
            )*
            temp_args
        }
    };
}