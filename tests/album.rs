use rocket::http::{ContentType, Status};
use rocket::local::Client;
use v::rocket;

#[test]
fn index() {
    let client = Client::new(v::rocket()).expect("valid rocket instance");

    let mut response = client.get("/a/").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::HTML));
    assert!(response.body().is_some());
}

#[test]
fn index_unsupported_methods() {
    let client = Client::new(v::rocket()).expect("valid rocket instance");
    let url = "/a/";

    for req in vec![
        client.delete(url),
        client.options(url),
        client.patch(url),
        client.post(url),
        client.put(url),
    ] {
        assert_eq!(req.dispatch().status(), Status::NotFound);
    }
}

#[test]
fn get_non_existent_token() {
    let client = Client::new(v::rocket()).expect("valid rocket instance");

    let mut response = dbg!(client.get("/a/this_token_does_not_exist").dispatch());

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.content_type(), Some(ContentType::Plain));
    assert_eq!(response.body_string(), Some("Could not find album".into()));
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
