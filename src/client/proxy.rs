use std::fmt::Display;


#[derive(Debug)]
pub enum ProxyType {
    Http,
    Https,
    #[cfg(feature = "socks-proxy")]
    SocksV5,
}

#[cfg(not(feature = "socks-proxy"))]
impl Display for ProxyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http => write!(f, "http://"),
            Self::Https => write!(f, "https://"),
        }
    }
}

#[cfg(feature = "socks-proxy")]
impl Display for ProxyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http => write!(f, "http://"),
            Self::Https => write!(f, "https://"),
            Self::SocksV5 => write!(f, "socks5://"),
        }
    }
}


#[derive(Debug)]
#[allow(dead_code)]
pub struct ClientProxy {
    server: String,
    proxy_type: ProxyType,
    port: Option<u16>,
}

impl ClientProxy {
    pub fn new(server: String, proxy_type: ProxyType, port: Option<u16>) -> Self {
        Self {
            server,
            proxy_type,
            port,
        }
    }

    pub(crate) fn into_reqwest_proxy(self) -> Result<reqwest::Proxy, ProxyConvertError> {
        let mut full_url = format!("{}{}", self.proxy_type, self.server);
        if let Some(port) = self.port {
            full_url.push_str(format!(":{}", port).as_str());
        }

        let proxy = reqwest::Proxy::all(full_url)?;

        Ok(proxy)
    }
}
