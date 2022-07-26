use std::net::SocketAddr;
use std::string::FromUtf8Error;
use axum::extract::Path;
use axum::Router;
use axum::routing::get;
use axum_odbc::{ODBCConnectionManager, OdbcManagerLayer};
use axum_odbc::odbc::{Bit, Cursor};
use axum_odbc::odbc::parameter::VarCharBox;

#[tokio::main]
async fn main() {
    let odbc_manager = ODBCConnectionManager::new("Driver={ODBC Driver 17 for SQL Server};Server=localhost;database=sql_training;UID=some_user;PWD=TEST;", 5);

    let app = Router::new()
        .route("/", get(root))
        .route("/country/:id", get(get_country))
        .route("/country", get(get_country))
        .layer(OdbcManagerLayer::new(odbc_manager));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello world!"
}

async fn get_country(manager: ODBCConnectionManager, country_id: Option<Path<i32>>) -> String {
    let connection = manager.aquire().await;
    if let Ok(connection) = connection {
        if let Some(Path(payload)) = country_id {
            if let Some(mut cursor) = connection.execute(format!("SELECT country_id, name, eu_member, description FROM country WHERE country_id={}", payload).as_str(), ()).ok().flatten() {
                if let Some(mut result) = cursor.next_row().ok().unwrap_or(None) {
                    let mut country_id: i32 = i32::default();
                    let mut name = Vec::<u8>::new();
                    let mut eu_member: Bit = Bit::default();
                    let mut description = Vec::<u8>::new();

                    let _ = result.get_data(1, &mut country_id);
                    let _ = result.get_text(2, &mut name);
                    let _ = result.get_data(3, &mut eu_member);
                    let _ = result.get_text(2, &mut description);

                    let name = match String::from_utf8(name) {
                        Ok(str) => str,
                        Err(_) => "".into()
                    };

                    let description = match String::from_utf8(description) {
                        Ok(str) => str,
                        Err(_) => "".into()
                    };

                    return format!("{{\"country_id\":{},\"name\":\"{}\",\"eu_member\":{},\"description\":\"{}\"}}", country_id, name, eu_member.as_bool(), description);
                } else {
                    return "{\"error\":\"No such country exists\"}".into();
                }
            } else {
                return "{\"data\":[]}".into();
            }
        } else {
            if let Some(mut cursor) = connection.execute("SELECT country_id, name, eu_member, description FROM country", ()).ok().flatten() {
                let mut data = Vec::<Country>::new();

                while let Some(mut result) = cursor.next_row().ok().unwrap_or(None) {
                    let mut country_id: i32 = i32::default();
                    let mut name = Vec::<u8>::new();
                    let mut eu_member: Bit = Bit::default();
                    let mut description = Vec::<u8>::new();

                    let _ = result.get_data(1, &mut country_id);
                    let _ = result.get_text(2, &mut name);
                    let _ = result.get_data(3, &mut eu_member);
                    let _ = result.get_text(2, &mut description);

                    let name = match String::from_utf8(name) {
                        Ok(str) => str,
                        Err(_) => "".into()
                    };

                    let description = match String::from_utf8(description) {
                        Ok(str) => str,
                        Err(_) => "".into()
                    };

                    data.push(Country {
                        country_id,
                        name,
                        eu_member: eu_member.as_bool(),
                        description
                    })
                }

                let mut ret = String::new();
                let mut first = true;

                for Country{country_id, name, eu_member, description} in &data {
                    if first {
                        ret = format!("{{\"country_id\":{},\"name\":\"{}\",\"eu_member\":{},\"description\":\"{}\"}}", country_id, name, eu_member, description);
                        first = false;
                    } else {
                        ret = ret + format!(",{{\"country_id\":{},\"name\":\"{}\",\"eu_member\":{},\"description\":\"{}\"}}", country_id, name, eu_member, description).as_str();
                    }
                }

                return format!("{{\"data\":[{}]}}", ret);
            } else {
                return "{\"data\":[]}".into();
            }
        }

    } else if let Err(error) = connection {
        return format!("{{ error: \"{:?}\" }}", error);
    }

    "".into()
}

struct Country {
    country_id: i32,
    name: String,
    eu_member: bool,
    description: String
}