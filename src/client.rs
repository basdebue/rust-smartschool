use crate::error::{Error, Result};
use futures::compat::Compat01As03;
use futures_01::stream::Stream;
use regex::Regex;
use reqwest::header::{HeaderMap, COOKIE, SET_COOKIE};
use reqwest::r#async::Client as HttpClient;
use reqwest::RedirectPolicy;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Client<'a> {
    http_client: HttpClient,
    session_id: String,
    url: &'a str,
}

impl<'a> Client<'a> {
    /// Gets an immutable reference to the underlying asynchronous HTTP client.
    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    /// Creates a client from login credentials.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let client = await!(Client::login(
    ///     "https://myschool.smartschool.be",
    ///     "username",
    ///     "password"
    /// ))
    /// .expect("error while logging in");
    /// ```
    pub async fn login(url: &'a str, username: &'a str, password: &'a str) -> Result<Client<'a>> {
        let http_client = HttpClient::builder()
            .redirect(RedirectPolicy::none())
            .build()?;

        let (session_id, token) = await!(get_session_id_and_token(&http_client, url))?;
        let session_id = await!(post_login_credentials(
            &http_client,
            url,
            &session_id,
            username,
            password,
            &token
        ))?;

        Ok(Client {
            http_client,
            session_id,
            url,
        })
    }

    /// Gets a slice of the current session's ID string.
    pub fn session_id(&self) -> &str {
        &self.session_id[..]
    }

    /// Gets a slice of the current session's URL string.
    pub fn url(&self) -> &str {
        self.url
    }
}

/// Extracts the session ID and login token from the login page.
async fn get_session_id_and_token<'a>(
    http_client: &'a HttpClient,
    url: &'a str,
) -> Result<(String, String)> {
    let url = format!("{}/login", url);
    let response = await!(Compat01As03::new(http_client.get(&url).send()))?;

    let session_id = get_session_id_cookie(response.headers())
        .ok_or(Error::Response("Server response did not contain PHPSESSID"))?
        .to_owned();

    let body = await!(Compat01As03::new(response.into_body().concat2()))?;
    let body = std::str::from_utf8(&body)
        .map_err(|_| Error::Response("Server response was not UTF-8-encoded"))?;

    let token = get_token(body)
        .ok_or(Error::Response("Server response did not contain token"))?
        .to_owned();

    Ok((session_id, token))
}

/// Extracts a new PHPSESSID cookie from response headers.
fn get_session_id_cookie(headers: &HeaderMap) -> Option<&str> {
    let re =
        Regex::new("PHPSESSID=(.+?);").expect("error while creating get_session_id_cookie regex");
    for header in headers.get_all(SET_COOKIE) {
        let cookie = header.to_str().ok()?;
        if let Some(captures) = re.captures(cookie) {
            return Some(captures.get(1)?.as_str());
        }
    }
    None
}

/// Extracts the login token from a response body.
fn get_token(body: &str) -> Option<&str> {
    // The token's <input> element happens to be the only one on the page that has two spaces before the `value` attribute.
    // If you get an error saying the token is missing, this regex probably doesn't work anymore.
    let re = Regex::new("  value=\"(.+?)\"").expect("error while creating get_token regex");
    if let Some(captures) = re.captures(body) {
        return Some(captures.get(1)?.as_str());
    }
    None
}

/// Posts login credentials in exchange for a session ID.
async fn post_login_credentials<'a>(
    http_client: &'a HttpClient,
    url: &'a str,
    session_id: &'a str,
    username: &'a str,
    password: &'a str,
    token: &'a str,
) -> Result<String> {
    let mut form = HashMap::new();
    form.insert("login_form[_password]", password);
    form.insert("login_form[_token]", token);
    form.insert("login_form[_username]", username);

    let url = format!("{}/login", url);
    let cookie = format!("PHPSESSID={}", session_id);
    let response = await!(Compat01As03::new(
        http_client
            .post(&url)
            .header(COOKIE, cookie)
            .form(&form)
            .send()
    ))?;

    // If the response doesn't contain a session ID, the login credentials are most likely invalid.
    let session_id = get_session_id_cookie(response.headers())
        .ok_or(Error::Response(
            "Invalid login credentials or expired login token",
        ))?
        .to_owned();

    Ok(session_id)
}
