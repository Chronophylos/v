use crate::{deletion_token::DeletionToken, models::Album, VDbConn};
use diesel::{OptionalExtension, RunQueryDsl};
use log::trace;
use rocket::{
    http::{RawStr, Status},
    request::FormDataError,
    request::FormError,
    response::status::Created,
    response::Redirect,
};
use rocket::{request::Form, response::status::Custom};
use rocket_contrib::templates::Template;
use serde::Serialize;
use std::collections::HashMap;

#[get("/")]
pub fn index() -> Template {
    trace!("handling GET /a/");

    Template::render("album/new", ())
}

#[get("/new")]
pub fn new() -> Redirect {
    trace!("handling GET /a/new");

    Redirect::to("/a")
}

#[derive(Debug, Serialize)]
pub struct AlbumContext {
    pub title: Option<String>,
    pub token: String,
    pub images: Vec<String>,
}

#[get("/<token>")]
pub fn get(conn: VDbConn, token: &RawStr) -> Result<Template, Custom<String>> {
    trace!("handling GET /a/{}", token);

    let album = Album::by_token(&token)
        .first::<Album>(&*conn)
        .optional()
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?
        .ok_or(Custom(Status::NotFound, "Could not find album".into()))?;

    let images = album
        .get_image_urls(&*conn)
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;

    Ok(Template::render(
        "album/show",
        AlbumContext {
            title: album.title,
            token: album.token,
            images,
        },
    ))
}

#[head("/<token>")]
pub fn head(conn: VDbConn, token: &RawStr) -> Result<(), Custom<String>> {
    trace!("handling HEAD /a/{}", token);

    Album::by_token(&token)
        .first::<Album>(&*conn)
        .optional()
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?
        .ok_or(Custom(Status::NotFound, "Could not find album".into()))?;

    Ok(())
}

#[derive(Debug, FromForm)]
pub struct NewAlbumForm {
    title: String,
    images: String,
}

#[post("/", data = "<sink>")]
pub fn post(
    conn: VDbConn,
    sink: Result<Form<NewAlbumForm>, FormError>,
) -> Result<Created<Template>, Custom<String>> {
    trace!("handling POST /a");

    let form_result = sink.map_err(|err| match err {
        FormDataError::Io(_) => Custom(
            Status::BadRequest,
            "Form input was invalid UTF-8.".to_string(),
        ),
        FormDataError::Malformed(f) | FormDataError::Parse(_, f) => {
            Custom(Status::BadRequest, format!("Invalid form input: {}", f))
        }
    })?;

    let title = if form_result.title.is_empty() {
        None
    } else {
        Some(form_result.title.as_str())
    };

    let album = Album::new(&*conn, title)
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;

    let mut images = Vec::new();

    for url in form_result.images.split(',') {
        let image = album
            .add_image(&*conn, url)
            .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;
        images.push(image);
    }

    // TODO: show album
    let mut context = HashMap::new();
    context.insert("deletion_token", album.deletion_token);
    context.insert("token", album.token.clone());
    Ok(Created(
        format!("/a/{}", album.token),
        Some(Template::render("album/created", &context)),
    ))
}

#[patch("/<token>")]
pub fn patch(_conn: VDbConn, token: &RawStr, _deletion_token: DeletionToken<'_>) -> Custom<()> {
    //let repo = Repo::borrow_from(&state).clone();

    trace!("handling PATCH /a/{}", token);

    Custom(Status::NotImplemented, ())
}
