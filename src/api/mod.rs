pub mod restaurant;
pub mod reservation;
pub mod table;
pub mod visual;

use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    reservation::routes(cfg);
    restaurant::routes(cfg);
    table::routes(cfg);   // << IMPORTANTE
    visual::routes(cfg);
}
