use bitcoind::bitcoincore_rpc::Client;
use bitcoind::{BitcoinD, ConnectParams};
use std::ffi::OsStr;
use std::fmt;

pub use bitcoind;
pub use bitcoind::bitcoincore_rpc;

mod versions;

pub struct ElementsD(BitcoinD);

#[non_exhaustive]
pub struct Conf<'a>(pub bitcoind::Conf<'a>);

/// All the possible error in this crate
pub enum Error {
    /// Wrapper of bitcoind Error
    BitcoinD(bitcoind::Error),
    /// Returned when calling methods requiring a feature to be activated, but it's not
    NoFeature,
    /// Returned when calling methods requiring a env var to exist, but it's not
    NoEnvVar,
    /// Returned when calling methods requiring either a feature or anv var, but both are present
    BothFeatureAndEnvVar,
    /// Returned when calling methods requiring the `elementsd` executable but none is found
    /// (no feature, no `ELEMENTSD_EXE`, no `elementsd` in `PATH` )
    NoElementsdExecutableFound,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BitcoinD(e) => write!(f, "{:?}", e),
            Error::NoFeature => write!(f, "Called a method requiring a feature to be set, but it's not"),
            Error::NoEnvVar => write!(f, "Called a method requiring env var `ELEMENTSD_EXE` to be set, but it's not"),
            Error::BothFeatureAndEnvVar => write!(f, "Called a method requiring env var `ELEMENTSD_EXE` or a feature to be set, but both are set"),
            Error::NoElementsdExecutableFound =>  write!(f, "Called a method requiring env var `ELEMENTSD_EXE` or a feature to be set or `elementsd` executable in path"),
        }
    }
}

impl Conf<'_> {
    pub fn new(validate_pegin: Option<&BitcoinD>) -> Self {
        let mut bitcoind_conf = bitcoind::Conf::default();
        let mut args = vec![
            "-fallbackfee=0.0001",
            "-dustrelayfee=0.00000001",
            "-chain=liquidregtest",
            "-initialfreecoins=2100000000",
        ];
        match validate_pegin.as_ref() {
            Some(bitcoind) => {
                args.push("-validatepegin=1");

                args.push(string_to_static_str(format!(
                    "-mainchainrpccookiefile={}",
                    bitcoind.params.cookie_file.display()
                )));
                args.push(string_to_static_str(format!(
                    "-mainchainrpchost={}",
                    bitcoind.params.rpc_socket.ip()
                )));
                args.push(string_to_static_str(format!(
                    "-mainchainrpcport={}",
                    bitcoind.params.rpc_socket.port()
                )));
            }
            None => {
                args.push("-validatepegin=0");
            }
        }

        bitcoind_conf.args = args;
        bitcoind_conf.network = "liquidregtest";

        Conf(bitcoind_conf)
    }
}

impl Default for Conf<'_> {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ElementsD {
    /// Launch the elementsd process from the given `exe` executable with default args.
    ///
    /// Waits for the node to be ready to accept connections before returning
    pub fn new<S: AsRef<OsStr>>(exe: S) -> Result<ElementsD, Error> {
        ElementsD::with_conf(exe, &Conf::default())
    }

    /// Launch the elementsd process from the given `exe` executable with given [Conf] param
    pub fn with_conf<S: AsRef<OsStr>>(exe: S, conf: &Conf) -> Result<ElementsD, Error> {
        Ok(ElementsD(BitcoinD::with_conf(exe, &conf.0)?))
    }

    pub fn client(&self) -> &Client {
        &self.0.client
    }

    pub fn params(&self) -> &ConnectParams {
        &self.0.params
    }

    pub fn rpc_url(&self) -> String {
        self.0.rpc_url()
    }

    pub fn rpc_url_with_wallet<T: AsRef<str>>(&self, wallet_name: T) -> String {
        self.0.rpc_url_with_wallet(wallet_name)
    }
}

/// Returns the daemons executable path, if it's provided as a feature or as `ELEMENTSD_EXE` env var
/// If both are set, the one provided by the feature is returned
pub fn exe_path() -> Result<String, Error> {
    match (downloaded_exe_path(), std::env::var("ELEMENTSD_EXE")) {
        (Ok(_), Ok(_)) => Err(Error::BothFeatureAndEnvVar),
        (Ok(path), Err(_)) => Ok(path),
        (Err(_), Ok(path)) => Ok(path),
        (Err(_), Err(_)) => which::which("elementsd")
            .map_err(|_| Error::NoElementsdExecutableFound)
            .map(|p| p.display().to_string()),
    }
}

/// Provide the bitcoind executable path if a version feature has been specified
pub fn downloaded_exe_path() -> Result<String, Error> {
    if versions::HAS_FEATURE {
        Ok(format!(
            "{}/elements/elements-{}/bin/elementsd",
            env!("OUT_DIR"),
            versions::VERSION
        ))
    } else {
        Err(Error::NoFeature)
    }
}

impl From<bitcoind::Error> for Error {
    fn from(e: bitcoind::Error) -> Self {
        Error::BitcoinD(e)
    }
}

//TODO remove this bad code once Conf::args is not Vec<&str>
fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

#[cfg(test)]
mod tests {
    use crate::{exe_path, Conf, ElementsD};
    use bitcoind::bitcoincore_rpc::jsonrpc::serde_json::Value;
    use bitcoind::bitcoincore_rpc::RpcApi;
    use bitcoind::BitcoinD;

    #[test]
    fn test_elementsd() {
        let exe = init();
        let elementsd = ElementsD::new(exe).unwrap();
        let info = elementsd
            .client()
            .call::<Value>("getblockchaininfo", &[])
            .unwrap();
        assert_eq!(info.get("chain").unwrap(), "liquidregtest");
    }

    #[test]
    fn test_elementsd_with_validatepegin() {
        let bitcoind_exe = bitcoind::exe_path().unwrap();
        let bitcoind_conf = bitcoind::Conf::default();
        let bitcoind = BitcoinD::with_conf(&bitcoind_exe, &bitcoind_conf).unwrap();
        let conf = Conf::new(Some(&bitcoind));
        let exe = init();
        let elementsd = ElementsD::with_conf(exe, &conf).unwrap();
        let info = elementsd
            .client()
            .call::<Value>("getblockchaininfo", &[])
            .unwrap();
        assert_eq!(info.get("chain").unwrap(), "liquidregtest");
    }

    #[test]
    fn test_rpc_url() {
        let exe = init();
        let elementsd = ElementsD::new(exe).unwrap();
        assert!(elementsd.rpc_url().starts_with("http://"));
        let socket = elementsd.params().rpc_socket.to_string();
        assert!(elementsd.rpc_url().ends_with(&socket));
        let url_with_wallet = elementsd.rpc_url_with_wallet("wallet_name");
        assert!(url_with_wallet.starts_with("http://"));
        assert!(url_with_wallet.ends_with("/wallet/wallet_name"));
    }

    fn init() -> String {
        let _ = env_logger::try_init();
        exe_path().unwrap()
    }
}
