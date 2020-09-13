use rocket::http::{ContentType, Status};
use rocket::local::Client;
use v::rocket;

const URL: &'static str = "/";

#[test]
fn get() {
    let client = Client::new(v::rocket()).expect("valid rocket instance");

    let mut response = client.get(URL).dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::HTML));
    assert!(response.body().is_some());
}

#[test]
fn head() {
    let client = Client::new(v::rocket()).expect("valid rocket instance");

    let mut response = client.head(URL).dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert!(response.content_type().is_none());
    assert!(response.body().is_none());
}

#[test]
fn feelsdankman() {
    let client = Client::new(v::rocket()).expect("valid rocket instance");

    let mut response = client.get("/FeelsDankMan").dispatch();

    assert_eq!(response.status(), Status::ImATeapot);
    assert_eq!(response.content_type(), Some(ContentType::Plain));
    assert_eq!(response.body_string(), Some("FeelsDankMan".to_string()));

    let mut response = client.head("/FeelsDankMan").dispatch();

    assert_eq!(response.status(), Status::ImATeapot);
    assert_eq!(response.content_type(), Some(ContentType::Plain));
    assert_eq!(response.body_string(), Some(String::new()));
}

#[test]
fn unsupported_methods() {
    let client = Client::new(v::rocket()).expect("valid rocket instance");

    for req in vec![
        client.delete(URL),
        client.options(URL),
        client.patch(URL),
        client.post(URL),
        client.put(URL),
    ] {
        assert_eq!(req.dispatch().status(), Status::NotFound);
    }
}
