use std::error::Error as StdError;

use paste::paste;
use snafu::prelude::*;
use snafu::IntoError;


macro_rules! gen_crypto_error_helpers {
    ($($sn_name:ident => $err_context:path),+ $(,)?) => {
        impl CryptoError {
            paste! {
                $(
                    pub fn [<new_ $sn_name _error>](msg: impl ToString) -> Self {
                        let inner = InnerError::from(msg.to_string());
                        $err_context.into_error(inner)
                    }

                    pub fn [<from_existing_to_ $sn_name>]<E, S>(err: E, msg: Option<S>) -> Self
                    where
                        E: StdError + Send + Sync + 'static,
                        S: ToString,
                    {
                        let msg = msg.map(|s| s.to_string());
                        let inner = InnerError::wrapped_error(err, msg);
                        $err_context.into_error(inner)
                    }
                )+
            }
        }

        pub trait IntoCryptoError {
            paste! {
                $(
                    fn [<to_ $sn_name _error>]<S: ToString>(self, msg: Option<S>) -> CryptoError;
                )+
            }
        }

        impl<E> IntoCryptoError for E
        where
            E: StdError + Send + Sync + 'static,
        {
            paste! {
                $(
                    fn [<to_ $sn_name _error>]<S: ToString>(self, msg: Option<S>) -> CryptoError {
                        CryptoError::[<from_existing_to_ $sn_name>](self, msg)
                    }
                )+
            }
        }

        pub trait ResultExt<T, E>
        where
            E: IntoCryptoError,
        {
            paste! {
                $(
                    fn [<map_error_ $sn_name>]<S: ToString>(self, msg: Option<S>) -> Result<T, CryptoError>;
                )+
            }
        }

        impl<T, E> ResultExt<T, E> for Result<T, E>
        where
            E: IntoCryptoError,
        {
            paste! {
                $(
                    fn [<map_error_ $sn_name>]<S: ToString>(self, msg: Option<S>) -> Result<T, CryptoError> {
                        self.map_err(|e| e.[<to_ $sn_name _error>](msg))
                    }
                )+
            }
        }
    };
}

pub type BoxedStdError = Box<dyn StdError + Send + Sync>;

#[derive(Debug, Snafu)]
#[snafu(module, context(suffix(false)))]
pub enum InnerError {
    #[snafu(display("{msg}"))]
    Msg {
        msg: String,
        backtrace: snafu::Backtrace,
    },

    #[snafu(display("{source:?}"))]
    Wrapped {
        source: BoxedStdError,
        backtrace: snafu::Backtrace,
    },

    #[snafu(display("{msg}"))]
    WrappedWithMsg {
        msg: String,
        source: BoxedStdError,
        backtrace: snafu::Backtrace,
    },
}

impl InnerError {
    fn wrapped_error<E>(err: E, msg: Option<String>) -> InnerError
    where
        E: StdError + Send + Sync + 'static,
    {
        let err: BoxedStdError = Box::new(err);

        match msg {
            Some(msg) => inner_error::WrappedWithMsg { msg }.into_error(err),
            None => inner_error::Wrapped.into_error(err),
        }
    }
}

impl From<String> for InnerError {
    fn from(value: String) -> Self {
        inner_error::Msg { msg: value }.build()
    }
}

/// The main error type that should be returned by any implementer of the
/// [AesDecryptor](crate::aes::AesDecryptor) and XXX traits
#[derive(Debug, Snafu)]
#[snafu(module, context(suffix(false)))]
pub enum CryptoError {
    #[snafu(display("Aes decryption error: {source}"))]
    AesDecrypt {
        #[snafu(backtrace)]
        source: InnerError,
    },

    #[snafu(display("Aes decryptor build error: {source}"))]
    AesDecryptorBuild {
        #[snafu(backtrace)]
        source: InnerError,
    },

    #[snafu(display("Private key generation error: {source}"))]
    PkeyGen {
        #[snafu(backtrace)]
        source: InnerError,
    },

    #[snafu(display("Private key decryption error: {source}"))]
    PkeyDecrypt {
        #[snafu(backtrace)]
        source: InnerError,
    },

    #[snafu(display("Public key encoding error: {source}"))]
    PkeyPubEncode {
        #[snafu(backtrace)]
        source: InnerError,
    },
}

gen_crypto_error_helpers! {
    aes_decrypt => crypto_error::AesDecrypt,
    aes_decryptor_build => crypto_error::AesDecryptorBuild,
    pkey_gen => crypto_error::PkeyGen,
    pkey_decrypt => crypto_error::PkeyDecrypt,
    pkey_pub_encode => crypto_error::PkeyPubEncode,
}
