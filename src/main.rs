use std::env;
use std::collections::HashMap;

use mysql::Conn;

use geo::{Polygon, Point};

use geo_raycasting::RayCasting;

use serde_json::value::Value;

use log::error;

struct City {
    id: u16,
    name: String,
    coords: Polygon<f64>,
}

fn convert_to_f64(input: &Value) -> Result<f64, ()> {
    match input {
        Value::String(s) => {
            if s.is_empty() {
                Err(())
            }
            else {
                s.parse().map_err(|e| error!("json value convert_to_f64 error: {}", e))
            }
        },
        Value::Number(n) => n.as_f64().ok_or_else(|| error!("json value convert_to_f64 error: json element isn't a float")),
        _ => {
            error!("json value convert_to_f64 format not recognized: {:?}", input);
            Err(())
        },
    }
}

fn main() -> Result<(), ()> {
    env_logger::init();

    let mut conn = Conn::new(env::var("DATABASE_URL").map_err(|_| error!("Missing env var DATABASE_URL"))?).map_err(|_| error!("Can't connect to MySQL"))?;
    let cities = {
        let res = conn.query("SELECT id, name, coordinates FROM city WHERE scadenza > UNIX_TIMESTAMP()").map_err(|e| error!("MySQL query error: {}", e))?;

        let mut temp = HashMap::new();
        for r in res {
            let mut row = r.map_err(|e| error!("MySQL row error: {}", e))?;

            let coords: String = row.take("coordinates").ok_or_else(|| error!("MySQL city.coordinates encoding error"))?;
            let mut poly: Vec<[f64; 2]> = Vec::new();
            for (i, c) in coords.replace("(", "").replace(")", "").split(",").enumerate() {
                let f: f64 = c.trim().parse().map_err(|e| error!("Coordinate parse error \"{}\": {}", c, e))?;
                if i % 2 == 0 {
                    poly.push([f, 0_f64]);
                }
                else {
                    let len = poly.len();
                    poly[len - 1][1] = f;
                }
            }

            let id = row.take("id").ok_or_else(|| error!("MySQL city.id encoding error"))?;
            temp.insert(id, City {
                id,
                name: row.take("name").ok_or_else(|| error!("MySQL city.name encoding error"))?,
                coords: Polygon::new(poly.into(), vec![]),
            });
        }
        temp
    };

    let res = conn.query("SELECT u.user_id, u.username, u.city_id, b.config FROM utenti u INNER JOIN utenti_config_bot b ON b.user_id = u.user_id WHERE u.status > 0").map_err(|e| error!("MySQL query error: {}", e))?;

    for r in res {
        let mut row = match r {
            Ok(row) => row,
            Err(e) => {
                error!("MySQL row error: {}", e);
                continue;
            }
        };

        let user_id: u64 = match row.take("user_id") {
            Some(user_id) => user_id,
            None => {
                error!("MySQL utenti.user_id encoding error");
                continue;
            }
        };
        let username: String = match row.take("username") {
            Some(username) => username,
            None => {
                error!("MySQL utenti.name encoding error");
                continue;
            }
        };
        let city_id: u16 = match row.take("city_id") {
            Some(city_id) => city_id,
            None => {
                error!("MySQL utenti.city_id encoding error");
                continue;
            }
        };
        if !cities.contains_key(&city_id) {
            println!("User @{} ({}) is assigned to disabled city {}", username, user_id, city_id);
            continue;
        }

        let config: String = match row.take("config") {
            Some(config) => config,
            None => {
                error!("MySQL utenti_config_bot.config encoding error");
                continue;
            }
        };

        let json: Value = match serde_json::from_str(&config) {
            Ok(json) => json,
            Err(e) => {
                error!("MySQL utenti_config_bot.config deserializing error: {}", e);
                continue;
            }
        };

        match (convert_to_f64(&json["locs"]["h"][0]), convert_to_f64(&json["locs"]["h"][1])) {
            (Ok(x), Ok(y)) => {
                let p: Point<f64> = (x, y).into();
                if !cities[&city_id].coords.within(&p) {
                    let mut not_found = true;
                    for (_, city) in &cities {
                        if city.coords.within(&p) {
                            println!("User @{} ({}) has home pointer in {} ({}) instead of {} ({})", username, user_id, city.name, city.id, cities[&city_id].name, city_id);
                            not_found = false;
                            break;
                        }
                    }
                    if not_found {
                        println!("User @{} ({}) has home pointer out of any known city", username, user_id);
                    }
                }
            },
            _ => {},
        }

        match (convert_to_f64(&json["locs"]["p"][0]), convert_to_f64(&json["locs"]["p"][1])) {
            (Ok(x), Ok(y)) => {
                let p: Point<f64> = (x, y).into();
                if !cities[&city_id].coords.within(&p) {
                    let mut not_found = true;
                    for (_, city) in &cities {
                        if city.coords.within(&p) {
                            println!("User @{} ({}) has pokémon pointer in {} ({}) instead of {} ({})", username, user_id, city.name, city.id, cities[&city_id].name, city_id);
                            not_found = false;
                            break;
                        }
                    }
                    if not_found {
                        println!("User @{} ({}) has pokémon pointer out of any known city", username, user_id);
                    }
                }
            },
            _ => {},
        }

        match (convert_to_f64(&json["locs"]["r"][0]), convert_to_f64(&json["locs"]["r"][1])) {
            (Ok(x), Ok(y)) => {
                let p: Point<f64> = (x, y).into();
                if !cities[&city_id].coords.within(&p) {
                    let mut not_found = true;
                    for (_, city) in &cities {
                        if city.coords.within(&p) {
                            println!("User @{} ({}) has raid pointer in {} ({}) instead of {} ({})", username, user_id, city.name, city.id, cities[&city_id].name, city_id);
                            not_found = false;
                            break;
                        }
                    }
                    if not_found {
                        println!("User @{} ({}) has raid pointer out of any known city", username, user_id);
                    }
                }
            },
            _ => {},
        }

        match (convert_to_f64(&json["locs"]["i"][0]), convert_to_f64(&json["locs"]["i"][1])) {
            (Ok(x), Ok(y)) => {
                let p: Point<f64> = (x, y).into();
                if !cities[&city_id].coords.within(&p) {
                    let mut not_found = true;
                    for (_, city) in &cities {
                        if city.coords.within(&p) {
                            println!("User @{} ({}) has invasion pointer in {} ({}) instead of {} ({})", username, user_id, city.name, city.id, cities[&city_id].name, city_id);
                            not_found = false;
                            break;
                        }
                    }
                    if not_found {
                        println!("User @{} ({}) has invasion pointer out of any known city", username, user_id);
                    }
                }
            },
            _ => {},
        }
    }

    Ok(())
}
