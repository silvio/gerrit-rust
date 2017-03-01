
use call;
use error::GGRError;
use error::GGRResult;
use error::GerritError;
use url;

const ENDPOINT: &'static str = "/config/server";

pub struct Config {
    call: call::Call,
}

impl Config {
    /// create a new `Config` instance
    pub fn new(url: &url::Url) -> Config {
        Config {
            call: call::Call::new(url),
        }
    }

    /// returns the gerrit version
    pub fn get_version(&self) -> GGRResult<String> {
        let path = format!("{}/version", ENDPOINT);
        match self.call.get(&path) {
            Ok(cr) => {
                if cr.ok() {
                    cr.convert::<String>()
                } else {
                    Err(GGRError::GerritApiError(GerritError::GerritApi(cr.status(), String::from_utf8(cr.get_body().unwrap()).unwrap())))
                }
            },
            Err(x) => {
                Err(GGRError::General(format!("call problem with: {} ({})", path, x)))
            }
        }
    }
}

#[test]
fn test_get_version() {
    let config = Config {
        call: call::Call::new(&url::Url::parse("http://localhost:8080").unwrap()),
    };

    assert_eq!("2.13.5", config.get_version().unwrap());
}
