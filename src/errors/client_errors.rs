use thiserror::Error;


#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Builder failed to generate the RSA private key")]
    RsaGen {
        #[from]
        source: super::RsaGenError,
    },

    #[error("Builder failed to build the client")]
    ClientBuild,

    #[error("Client failed to register with the Interactsh server")]
    Register,

    #[error("Client failed to deregister with the Interactsh server")]
    Deregister,

    #[error("Client failed to poll the Interactsh server")]
    Poll,
}
