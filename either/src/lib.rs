pub const VERSION: &str = "0.0.1";

#[macro_export]
macro_rules! either {
    (
        $(#[$meta:meta])*
        $pub:vis $name:ident
        $(< $($lt:lifetime),* >)?
        $(, $alt:ident $(< $($alt_lt:lifetime),* >)? )*
    ) => {
        $(#[$meta])*
        $pub enum $name
        $(<$($lt),*>)?
        {
            $( $alt( $alt $(<$($alt_lt),*>)? ) ),*
        }
        $(
        impl
            $(< $($alt_lt),* >)?
        From< $alt
            $(< $($alt_lt),* >)?
        >
        for $name
            $(< $($alt_lt),* >)?
        {
            fn from(v: $alt
                $(< $($alt_lt),* >)?
            ) -> Self
            {
                Self::$alt(v)
            }
        }

        impl
            $(< $($alt_lt),* >)?
        std::convert::TryFrom< $name
            $(< $($alt_lt),* >)?
        >
        for $alt
            $(< $($alt_lt),* >)?
        {
            type Error = $name
                $(< $($alt_lt),* >)?
            ;
            fn try_from(a: $name
                $(< $($alt_lt),* >)?
            ) -> Result<Self, Self::Error>
            {
                match a {
                    $name::$alt(v) => Ok(v),
                    v => Err(v)
                }
            }
        }
        )*
    };
}

#[macro_export]
macro_rules! name {
    (
        $(#[$meta:meta])*
        $pub:vis $name:ident
        $(< $($lt:lifetime),* >)?
        = $t:ty
    ) => {
        $(#[$meta])*
        $pub struct $name
        $(<$($lt),*>)?
        (
            pub $t
        );
        impl
            $(<$($lt),*>)?
        From<$t> for $name
            $(<$($lt),*>)?
        {
            fn from(v: $t) -> Self {
                Self(v)
            }
        }
    };
}
