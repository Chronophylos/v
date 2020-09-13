use crate::{
    config::Config, deletion_token::DeletionToken, imgur::get_album_images, models::Album, VDbConn,
};
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
use std::collections::{HashMap, HashSet};
use url::Url;

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
    Album::by_token(&token)
        .first::<Album>(&*conn)
        .optional()
        .map_err(|_| Custom(Status::InternalServerError, ()))?
        .ok_or(Custom(Status::NotFound, ()))?;

    Ok(())
}

#[post("/<_token>/edit")]
pub fn edit(_conn: VDbConn, _token: &RawStr) -> Result<Template, Custom<String>> {
    todo!()
}

#[derive(Debug, FromForm)]
pub struct NewAlbumForm {
    title: String,
    images: String,
}

#[post("/new", data = "<sink>")]
pub fn new(
    conn: VDbConn,
    sink: Result<Form<NewAlbumForm>, FormError>,
    config: State<Config>,
) -> Result<Created<Template>, Custom<String>> {
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
        let url = validate_url(&config.allowed_domains, url)?;

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

#[derive(Debug, FromForm)]
pub struct ImportAlbumForm {
    title: String,
    link: String,
}

#[post("/import", data = "<sink>")]
pub fn import(
    conn: VDbConn,
    sink: Result<Form<ImportAlbumForm>, FormError>,
) -> Result<Created<Template>, Custom<String>> {
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

    let url: Url = form_result
        .link
        .parse()
        .map_err(|err| Custom(Status::BadRequest, format!("Invalid form input: {}", err)))?;

    if url.domain() != Some("imgur.com") {
        return Err(Custom(
            Status::BadRequest,
            format!("Invalid form input: URL is not allowed: {}", url),
        ));
    }

    let mut path_segments = url
        .path_segments()
        .ok_or_else(|| "cannot be base")
        .map_err(|err| {
            Custom(
                Status::BadRequest,
                format!("Could not parse album link: {}", err),
            )
        })?;
    let album_hash = path_segments.nth(1).unwrap();

    let links = get_album_images("", album_hash).map_err(|err| {
        Custom(
            Status::BadRequest,
            format!("Could not get album images: {}", err),
        )
    })?;

    let album = Album::new(&*conn, title)
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;

    let mut images = Vec::new();

    for link in links {
        let url: Url = link
            .parse()
            .map_err(|err| Custom(Status::BadRequest, format!("Invalid form input: {}", err)))?;

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

/// TODO: check content type
fn validate_image(_url: &Url) -> bool {
    true
}

pub fn validate_url(allowed_domains: &HashSet<String>, url: &str) -> Result<Url, Custom<String>> {
    let mut url: Url = url
        .parse()
        .map_err(|err| Custom(Status::BadRequest, format!("Invalid form input: {}", err)))?;

    match url.domain() {
        Some(domain) => {
            if !allowed_domains.contains(&domain.to_lowercase()) {
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
