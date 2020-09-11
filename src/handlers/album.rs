use crate::{deletion_token::DeletionToken, models::Album, DomainAllowList, VDbConn};
use anyhow::Result;
use diesel::{OptionalExtension, RunQueryDsl};
use log::trace;
use rocket::{
    http::{RawStr, Status},
    request::{Form, FormDataError, FormError},
    response::{status::Created, status::Custom},
    State,
};
use rocket_contrib::templates::Template;
use serde::Serialize;
use std::collections::HashMap;
use url::Url;

#[get("/")]
pub fn index() -> Template {
    trace!("handling GET /a/");

    Template::render("album/new", ())
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
pub fn head(conn: VDbConn, token: &RawStr) -> Result<(), Custom<()>> {
    trace!("handling HEAD /a/{}", token);

    Album::by_token(&token)
        .first::<Album>(&*conn)
        .optional()
        .map_err(|_| Custom(Status::InternalServerError, ()))?
        .ok_or(Custom(Status::NotFound, ()))?;

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
    domain_allow_list: State<DomainAllowList>,
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
        let url = validate_url(&domain_allow_list, url)?;

        if validate_image(&url) {
            let image = album
                .add_image(&*conn, url.as_str())
                .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;
            images.push(image);
        }
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
    trace!("handling PATCH /a/{}", token);

    Custom(Status::NotImplemented, ())
}

/// TODO: check content type
fn validate_image(_url: &Url) -> bool {
    true
}

pub fn validate_url(
    domain_allow_list: &State<DomainAllowList>,
    url: &str,
) -> Result<Url, Custom<String>> {
    let mut url: Url = url
        .parse()
        .map_err(|err| Custom(Status::BadRequest, format!("Invalid form input: {}", err)))?;

    match url.domain() {
        Some(domain) => {
            if !domain_allow_list.contains(&domain.to_lowercase()) {
                return Err(Custom(
                    Status::BadRequest,
                    format!("Invalid form input: URL is not allowed: {}", url),
                ));
            }
        }
        None => {
            return Err(Custom(
                Status::BadRequest,
                format!("Invalid form input: URL has no domain part: {}", url),
            ));
        }
    }

    if url.cannot_be_a_base() {
        return Err(Custom(
            Status::BadRequest,
            format!("Invalid form input: Not a base URL: {}", url),
        ));
    }

    match url.scheme() {
        "http" => url.set_scheme("http").unwrap(),
        "https" | "ftp" => {}
        _ => {
            return Err(Custom(
                Status::BadRequest,
                format!("Invalid form input: Invalid scheme: {}", url),
            ))
        }
    }

    Ok(url)
}
