use rocket::http::{ContentType, Status};
use rocket::local::Client;
use v::rocket;

#[test]
fn new() {
    let client = Client::new(v::rocket()).expect("valid rocket instance");

    let mut response = client
        .post("/a/new")
        .header(ContentType::Form)
        .body("title=title&images=https%3A%2F%2Fi.imgur.com%2FVoyouQH.png")
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    assert_eq!(response.content_type(), Some(ContentType::HTML));
    assert!(response.body().is_some());
}

#[test]
fn new_invalid_image_url() {
    let client = Client::new(v::rocket()).expect("valid rocket instance");

    let mut response = client
        .post("/a/new")
        .header(ContentType::Form)
        .body("title=title&images=https%3A%2F%2Fi.evil.com%2FVoyouQH.png")
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(response.content_type(), Some(ContentType::HTML));
    assert!(response.body().is_some());
}

#[test]
fn import() {
    let client = Client::new(v::rocket()).expect("valid rocket instance");

    let mut response = client
        .post("/a/new")
        .header(ContentType::Form)
        .body("title=&link=https%3A%2F%2Fimgur.com%2Fa%2FJrheYnV")
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    assert_eq!(response.content_type(), Some(ContentType::HTML));
    assert!(response.body().is_some());
}

#[test]
fn get_non_existent_token() {
    let client = Client::new(v::rocket()).expect("valid rocket instance");

    let mut response = dbg!(client.get("/a/this_token_does_not_exist").dispatch());

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.content_type(), Some(ContentType::Plain));
    assert_eq!(
        response.body_string(),
        Some(String::from("Could not find album"))
    );
}

#[test]
fn head_non_existent_token() {
    let client = Client::new(v::rocket()).expect("valid rocket instance");

    let mut response = dbg!(client.head("/a/this_token_does_not_exist").dispatch());

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.content_type(), None);
    assert_eq!(response.body_string(), None);
}

#[test]
fn token_unsupported_methods() {
    let client = Client::new(v::rocket()).expect("valid rocket instance");
    let url = "/a/this_token_does_not_exist";

    for req in vec![client.delete(url), client.options(url), client.put(url)] {
        assert_eq!(req.dispatch().status(), Status::NotFound);
    }
}
