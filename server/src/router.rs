use ntex::web;
use ntex::web::DefaultError;

mod authenticated;
mod handshake;
mod healthcheck;
mod pick_choice;
mod public_key;
mod verify;

pub(crate) fn routes() -> web::Scope<DefaultError> {
    web::scope("/").service((
        healthcheck::routes(),
        handshake::routes(),
        public_key::routes(),
        pick_choice::routes(),
        verify::routes(),
        authenticated::routes(),
    ))
}
