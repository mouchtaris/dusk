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
            ) -> std::result::Result<Self, Self::Error>
            {
                match a {
                    $name::$alt(v) => Ok(v),
                    v => Err(v)
                }
            }
        }

        impl
            < 'a, $( $($alt_lt),* )? >
        std::convert::TryFrom< &'a mut $name
            $(< $($alt_lt),* >)?
        >
        for &'a mut $alt
            $(< $($alt_lt),* >)?
        {
            type Error = &'a mut $name
                $(< $($alt_lt),* >)?
            ;
            fn try_from(a: &'a mut $name
                $(< $($alt_lt),* >)?
            ) -> std::result::Result<Self, Self::Error>
            {
                match a {
                    $name::$alt(v) => Ok(v),
                    v => Err(v)
                }
            }
        }

        impl
            < 'a, $( $($alt_lt),* )? >
        std::convert::TryFrom< &'a $name
            $(< $($alt_lt),* >)?
        >
        for &'a $alt
            $(< $($alt_lt),* >)?
        {
            type Error = &'a $name
                $(< $($alt_lt),* >)?
            ;
            fn try_from(a: &'a $name
                $(< $($alt_lt),* >)?
            ) -> std::result::Result<Self, Self::Error>
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

#[cfg(test)]
mod test {
    struct A;
    struct B;
    either![E, A, B];

    #[test]
    fn impl_from() {
        let _: E = A.into();
        let _: E = B.into();
    }

    #[test]
    fn impl_try_into() {
        use std::convert::TryFrom;
        let a: fn() -> E = || A.into();
        let b: fn() -> E = || B.into();

        assert!(A::try_from(a()).is_ok());
        assert!(B::try_from(b()).is_ok());
        assert!(A::try_from(b()).is_err());
        assert!(B::try_from(a()).is_err());

        assert!(<&mut A>::try_from(&mut a()).is_ok());
        assert!(<&mut B>::try_from(&mut b()).is_ok());
        assert!(<&mut A>::try_from(&mut b()).is_err());
        assert!(<&mut B>::try_from(&mut a()).is_err());

        assert!(<&A>::try_from(&a()).is_ok());
        assert!(<&B>::try_from(&b()).is_ok());
        assert!(<&A>::try_from(&b()).is_err());
        assert!(<&B>::try_from(&a()).is_err());
    }
}
