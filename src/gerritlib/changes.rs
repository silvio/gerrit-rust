
use call;
use entities;
use error::GGRError;
use error::GGRResult;
use rustc_serialize;


pub struct Changes { }

impl Changes {
    pub fn query_changes(call: &call::Call, querystring: &str) -> GGRResult<entities::ChangeInfos> {
        if let Ok(cr) = call.get("/changes/".into(), &querystring) {
            let body = match cr.body {
                Some(x) => x,
                None => {
                    /* no body content */
                    return Ok(entities::ChangeInfos::new());
                }
            };
            let data2 = body.iter().fold(String::from(""), |news, el| format!("{}{}", news, el));

            let data5 = match  rustc_serialize::json::Json::from_str(&data2) {
                Ok(d) => d,
                Err(e) => {
                    println!("error: {}",e);
                    return Err(GGRError::from(e));
                }
            };

            let changeinfos = entities::ChangeInfos::new_with_data(Some(data5));

            return Ok(changeinfos);
        } else {
            println!("call problem with: {}", &querystring);
        }
        Ok(entities::ChangeInfos::new())
    }
}

