use crate::{config::Config, imgur::get_album_images, models::Album, VDbConn};
use anyhow::Result;
use diesel::{OptionalExtension, PgConnection, RunQueryDsl};
use rocket::{
    data::FromData,
    http::Cookie,
    http::{Cookies, RawStr, Status},
    request::{Form, FormDataError, FormError},
    response::{status::Created, status::Custom, Redirect},
    State,
};
use rocket_contrib::templates::Template;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use url::Url;

#[derive(Debug, Serialize)]
pub struct AlbumContext<'a> {
    pub title: &'a Option<String>,
    pub token: &'a str,
    pub images: &'a Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AlbumSecretContext<'a> {
    pub title: &'a Option<String>,
    pub token: &'a str,
    pub deletion_token: &'a str,
    pub images: &'a Vec<String>,
}

#[get("/<token>")]
pub fn get(conn: VDbConn, token: &RawStr) -> Result<Template, Custom<String>> {
    let album = get_album(&conn, token)?;
    let images = get_image_urls(&conn, &album)?;

    Ok(Template::render(
        "album/show",
        /*
        AlbumContext {
            title: &album.title,
            token: &album.token,
            images: &images,
        },
        */
        AlbumSecretContext {
            title: &album.title,
            token: &album.token,
            deletion_token: &album.deletion_token,
            images: &images,
        },
    ))
}

#[head("/<token>")]
pub fn head(conn: VDbConn, token: &RawStr) -> Result<(), Custom<String>> {
    get_album(&conn, token)?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct AuthContext<'a> {
    pub title: &'a Option<String>,
    pub token: &'a str,
}

#[get("/<token>/auth")]
pub fn get_auth(conn: VDbConn, token: &RawStr) -> Result<Template, Custom<String>> {
    let album = get_album(&conn, token)?;

    Ok(Template::render(
        "album/auth",
        AuthContext {
            title: &album.title,
            token: &album.token,
        },
    ))
}

#[derive(Debug, FromForm)]
pub struct AuthForm {
    deletion_token: String,
}

#[post("/<token>/auth", data = "<sink>")]
pub fn post_auth(
    conn: VDbConn,
    token: &RawStr,
    sink: Result<Form<AuthForm>, FormError>,
    mut cookies: Cookies,
) -> Result<Redirect, Custom<String>> {
    let form_result = parse_form(sink)?;
    let album = get_album(&conn, token)?;

    check_deletion_token(&album, &form_result.deletion_token)?;

    let cookie = Cookie::new(
        token.percent_decode_lossy().to_string(),
        form_result.deletion_token.clone(),
    );

    cookies.add_private(cookie);

    Ok(Redirect::to(format!("/a/{}/edit", token)))
}

#[derive(Debug, FromForm)]
pub struct EditAlbumForm {
    index: u16,
    url: String,
    deletion_token: String,
    method: String,
}

#[get("/<token>/edit")]
pub fn get_edit(
    conn: VDbConn,
    token: &RawStr,
    mut cookies: Cookies,
) -> Result<Template, Custom<String>> {
    let album = get_album(&conn, token)?;

    check_deletion_token_cookie(&album, &mut cookies)?;

    let images = get_image_urls(&conn, &album)?;

    Ok(Template::render(
        "album/edit",
        AlbumSecretContext {
            title: &album.title,
            token: &album.token,
            deletion_token: &album.deletion_token,
            images: &images,
        },
    ))
}

#[post("/<token>/edit", data = "<sink>")]
pub fn post_edit(
    conn: VDbConn,
    token: &RawStr,
    sink: Result<Form<EditAlbumForm>, FormError>,
) -> Result<Template, Custom<String>> {
    let form_result = parse_form(sink)?;
    let album = get_album(&conn, token)?;

    check_deletion_token(&album, &form_result.deletion_token)?;

    match form_result.method.as_str() {
        "insert" => insert_image(&conn, &album, form_result.into_inner())?,
        _ => {
            return Err(Custom(
                Status::BadRequest,
                format!("Invalid method `{}`", form_result.method),
            ))
        }
    };

    let images = get_image_urls(&conn, &album)?;

    Ok(Template::render(
        "album/edit",
        AlbumSecretContext {
            title: &album.title,
            token: &album.token,
            deletion_token: &album.deletion_token,
            images: &images,
        },
    ))
}

#[derive(Debug, FromForm)]
pub struct NewAlbumForm {
    title: String,
    url: String,
}

#[post("/new", data = "<sink>")]
pub fn new(
    conn: VDbConn,
    sink: Result<Form<NewAlbumForm>, FormError>,
    config: State<Config>,
) -> Result<Created<Template>, Custom<String>> {
    let form_result = parse_form(sink)?;

    let title = if form_result.title.is_empty() {
        None
    } else {
        Some(form_result.title.as_str())
    };

    let album = Album::new(&*conn, title)
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;

    let url = validate_url(&config.allowed_domains, &form_result.url)?;

    album
        .add_image(&*conn, url.as_str(), 0)
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;

    // TODO: show album
    let mut context = HashMap::new();
    context.insert("deletion_token", album.deletion_token);
    context.insert("token", album.token.clone());
    Ok(Created(
        format!("/a/{}", album.token),
        Some(Template::render("album/created", &context)),
    ))
}

#[derive(Debug, FromForm)]
pub struct ImportAlbumForm {
    title: String,
    url: String,
}

#[post("/import", data = "<sink>")]
pub fn import(
    conn: VDbConn,
    sink: Result<Form<ImportAlbumForm>, FormError>,
    config: State<Config>,
) -> Result<Created<Template>, Custom<String>> {
    let form_result = parse_form(sink)?;

    let title = if form_result.title.is_empty() {
        None
    } else {
        Some(form_result.title.as_str())
    };

    let url: Url = form_result
        .url
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

    let links = get_album_images(&config.imgur.client_id, album_hash).map_err(|err| {
        Custom(
            Status::BadRequest,
            format!("Could not get album images: {}", err),
        )
    })?;

    let album = Album::new(&*conn, title)
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;

    let mut images = Vec::new();

    if links.len() > i32::MAX as usize {
        return Err(Custom(
            Status::BadRequest,
            format!("Album has too many images (more than {})", i32::MAX),
        ));
    }

    for (index, link) in links.into_iter().enumerate() {
        let url = parse_url(&link)?;

        if validate_image(&url) {
            let image = album
                .add_image(&*conn, url.as_str(), index as i32)
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

fn parse_form<'a, F>(sink: Result<F, FormError>) -> Result<F, Custom<String>>
where
    F: FromData<'a>,
{
    sink.map_err(|err| match err {
        FormDataError::Io(_) => Custom(
            Status::BadRequest,
            "Form input was invalid UTF-8.".to_string(),
        ),
        FormDataError::Malformed(f) | FormDataError::Parse(_, f) => {
            Custom(Status::BadRequest, format!("Invalid form input: {}", f))
        }
    })
}

fn get_album(conn: &PgConnection, token: &str) -> Result<Album, Custom<String>> {
    Album::by_token(token)
        .first::<Album>(conn)
        .optional()
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?
        .ok_or(Custom(Status::NotFound, "Could not find album".into()))
}

fn get_image_urls(conn: &PgConnection, album: &Album) -> Result<Vec<String>, Custom<String>> {
    album
        .get_image_urls(conn)
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))
}

fn check_deletion_token(album: &Album, deletion_token: &str) -> Result<(), Custom<String>> {
    (album.deletion_token != deletion_token.trim())
        .then_some(())
        .ok_or(Custom(
            Status::Forbidden,
            "Wrong deletion token".to_string(),
        ))
}

fn check_deletion_token_cookie(album: &Album, cookies: &mut Cookies) -> Result<(), Custom<String>> {
    check_deletion_token(
        album,
        cookies
            .get_private(&album.token)
            .ok_or(Custom(
                Status::Unauthorized,
                "Missing deletion token".to_string(),
            ))?
            .value(),
    )
}

fn parse_url(url: &str) -> Result<Url, Custom<String>> {
    url.parse()
        .map_err(|err| Custom(Status::BadRequest, format!("Invalid form input: {}", err)))
}

fn insert_image(
    conn: &PgConnection,
    album: &Album,
    form: EditAlbumForm,
) -> Result<(), Custom<String>> {
    let image_count = album
        .image_count(conn)
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;
    let url = parse_url(&form.url)?;

    if image_count > form.index as usize {
        album
            .increase_index(conn, form.index as i32)
            .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;
    }

    album
        .add_image(&*conn, url.as_str(), form.index as i32)
        .map_err(|err| Custom(Status::InternalServerError, err.to_string()))?;

    Ok(())
}
