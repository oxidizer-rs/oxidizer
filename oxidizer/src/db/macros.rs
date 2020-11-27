#[macro_export]
macro_rules! args {
    ( $( $x:expr),* ) => {
        {
            use oxidizer::db::types::*;
            &[ $( (&$x).to_db_type() ),*  ]
        }
    };
}
