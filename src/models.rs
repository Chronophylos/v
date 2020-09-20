use super::schema::{albums, images};
use anyhow::{Context, Result};
use diesel::{
    insert_into, pg::Pg, update, BelongingToDsl, ExpressionMethods, PgConnection, QueryDsl,
    RunQueryDsl,
};
use rand::seq::SliceRandom;

/// generate a token, deletion-token pair
/// The first token is 8 chars long and the second 16
pub fn generate_token_pair() -> (String, String) {
    let mut rng = rand::thread_rng();
    let alphabet = "ABCDEFGHIKJLMNOPQRSTUVWXYZabcdefghikjlmnopqrstuvwxyz0123456789"
        .to_string()
        .into_bytes();
    (
        alphabet
            .choose_multiple(&mut rng, 8)
            .map(|&b| b as char)
            .collect(),
        alphabet
            .choose_multiple(&mut rng, 16)
            .map(|&b| b as char)
            .collect(),
    )
}

#[derive(Debug, Queryable, Identifiable)]
pub struct Album {
    pub id: i32,

    pub token: String,
    pub deletion_token: String,

    pub title: Option<String>,
}

impl Album {
    pub fn new(conn: &PgConnection, title: Option<&str>) -> Result<Album> {
        let (token, deletion_token) = generate_token_pair();

        let new_album = NewAlbum {
            token: token.as_str(),
            deletion_token: deletion_token.as_str(),
            title,
        };

        insert_into(albums::table)
            .values(&new_album)
            .get_result(conn)
            .context("Could not insert new album")
    }

    pub fn by_token<'a>(token: &'a str) -> albums::BoxedQuery<'a, Pg> {
        albums::table.filter(albums::token.eq(token)).into_boxed()
    }

    pub fn by_deletion_token<'a>(token: &'a str) -> albums::BoxedQuery<'a, Pg> {
        albums::table
            .filter(albums::deletion_token.eq(token))
            .into_boxed()
    }

    pub fn add_image(&self, conn: &PgConnection, url: &str, index: i32) -> Result<Image> {
        Image::new(conn, self.id, url, index)
    }

    pub fn select_images<'a>(&'a self) -> images::BoxedQuery<'a, Pg> {
        Image::belonging_to(self).into_boxed()
    }

    pub fn get_image_urls(&self, conn: &PgConnection) -> Result<Vec<String>> {
        self.select_images()
            .select(images::url)
            .get_results(conn)
            .context("Could not get images belonging to album")
    }

    pub fn increase_index(&self, conn: &PgConnection, start: i32) -> Result<()> {
        update(Image::belonging_to(self).filter(images::index.ge(start)))
            .set(images::index.eq(images::index + 1))
            .execute(conn)?;
        Ok(())
    }

    pub fn image_count(&self, conn: &PgConnection) -> Result<usize> {
        Ok(self.select_images().count().get_result::<i64>(conn)? as usize)
    }
}

#[derive(Debug, Insertable)]
#[table_name = "albums"]
pub struct NewAlbum<'a> {
    pub token: &'a str,
    pub deletion_token: &'a str,

    pub title: Option<&'a str>,
}

#[derive(Debug, Queryable, Identifiable, Associations)]
#[belongs_to(Album, foreign_key = "album_id")]
pub struct Image {
    pub id: i32,
    pub album_id: i32,

    pub token: String,
    pub deletion_token: String,

    pub url: String,
    pub index: i32,
}

impl Image {
    pub fn new(conn: &PgConnection, album_id: i32, url: &str, index: i32) -> Result<Image> {
        let (token, deletion_token) = generate_token_pair();

        insert_into(images::table)
            .values(&NewImage {
                album_id,
                token: token.as_str(),
                deletion_token: deletion_token.as_str(),
                index,
                url,
            })
            .get_result(conn)
            .context("Could not insert new image")
    }

    pub fn by_token<'a>(token: &'a str) -> images::BoxedQuery<'a, Pg> {
        images::table.filter(images::token.eq(token)).into_boxed()
    }

    pub fn by_deletion_token<'a>(token: &'a str) -> images::BoxedQuery<'a, Pg> {
        images::table
            .filter(images::deletion_token.eq(token))
            .into_boxed()
    }
}

#[derive(Debug, Insertable)]
#[table_name = "images"]
pub struct NewImage<'a> {
    pub album_id: i32,

    pub token: &'a str,
    pub deletion_token: &'a str,

    pub url: &'a str,
    pub index: i32,
}
