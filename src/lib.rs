use std::ffi::OsStr;
use bitcoind::bitcoincore_rpc::Client;
use bitcoind::BitcoinD;

mod versions;

pub struct ElementsD(BitcoinD);

#[non_exhaustive]
pub struct Conf<'a>(bitcoind::Conf<'a>);

/// All the possible error in this crate
#[derive(Debug)]
pub enum Error {
    /// Wrapper of bitcoind Error
    BitcoinD(bitcoind::Error),
}

impl Conf<'_> {
    pub fn new(validate_pegin: Option<&BitcoinD>) -> Self {
        let mut bitcoind_conf = bitcoind::Conf::default();
        let mut args = vec!["-fallbackfee=0.0001", "-dustrelayfee=0.00000001", "-chain=liquidregtest", "-initialfreecoins=2100000000"];
        match validate_pegin.as_ref() {
            Some(bitcoind) => {
                args.push("-validatepegin=1");
                /*let mut f = File::open(&bitcoind.params.cookie_file).unwrap();
                let mut buffer = String::new();
                f.read_to_string(&mut buffer).unwrap();
                let vec: Vec<_> = buffer.split(":").collect();
                println!("cookie user:{} value:{}", vec[0], vec[1]);
                args.push(string_to_static_str(format!("-mainchainrpcuser={}", vec[0])));
                args.push(string_to_static_str(format!("-mainchainrpcpassword={}", vec[1])));*/
                args.push(string_to_static_str(format!("-mainchainrpccookiefile={}", bitcoind.params.cookie_file.display())));
                args.push(string_to_static_str(format!("-mainchainrpchost={}", bitcoind.params.rpc_socket.ip())));
                args.push(string_to_static_str(format!("-mainchainrpcport={}", bitcoind.params.rpc_socket.port())));
            }
            None => {
                args.push("-validatepegin=0" );
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
}

/// Provide the bitcoind executable path if a version feature has been specified
pub fn downloaded_exe_path() -> Option<String> {
    if versions::HAS_FEATURE {
        Some(format!(
            "{}/elements/elements-{}/bin/elementsd",
            env!("OUT_DIR"),
            versions::VERSION
        ))
    } else {
        None
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
    use std::env;
    use bitcoind::bitcoincore_rpc::jsonrpc::serde_json::Value;
    use crate::{Conf, downloaded_exe_path, ElementsD};
    use bitcoind::bitcoincore_rpc::RpcApi;
    use bitcoind::BitcoinD;

    #[test]
    fn test_elementsd() {
        let exe = init();
        let elementsd = ElementsD::new(exe).unwrap();
        let info = elementsd.client().call::<Value>("getblockchaininfo", &[]).unwrap();
        assert_eq!( info.get("chain").unwrap(), "liquidregtest");
    }

    #[test]
    fn test_elementsd_with_validatepegin() {
        let bitcoind_exe = bitcoind_exe_path();
        let bitcoind_conf = bitcoind::Conf::default();
        let bitcoind = BitcoinD::with_conf(&bitcoind_exe, &bitcoind_conf).unwrap();
        let conf = Conf::new(Some(&bitcoind));
        let exe = init();
        let elementsd = ElementsD::with_conf(exe, &conf).unwrap();
        let info = elementsd.client().call::<Value>("getblockchaininfo", &[]).unwrap();
        assert_eq!( info.get("chain").unwrap(), "liquidregtest");
    }

    fn exe_path() -> String {
        if let Some(downloaded_exe_path) = downloaded_exe_path() {
            downloaded_exe_path
        } else {
            env::var("ELEMENTSD_EXE").expect(
                "when no version feature is specified, you must specify ELEMENTSD_EXE env var",
            )
        }
    }

    fn bitcoind_exe_path() -> String {
        if let Some(downloaded_exe_path) = bitcoind::downloaded_exe_path() {
            downloaded_exe_path
        } else {
            env::var("BITCOINDD_EXE").expect(
                "when no version feature is specified, you must specify BITCOIND_EXE env var",
            )
        }
    }


    fn init() -> String {
        let _ = env_logger::try_init();
        exe_path()
    }
}
