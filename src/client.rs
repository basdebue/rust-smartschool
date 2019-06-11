//! A client for interacting with a Smartschool instance.

use crate::error::{Error, Result};
use futures::compat::{Future01CompatExt, Stream01CompatExt};
use futures::stream::TryStreamExt;
use regex::Regex;
use reqwest::r#async::Client as HttpClient;
use reqwest::RedirectPolicy;
use std::collections::HashMap;

/// A struct containing authentication data and an asynchronous HTTP client.
#[derive(Clone, Debug)]
pub struct Client<'a> {
    http_client: HttpClient,
    url: &'a str,
}

impl<'a> Client<'a> {
    /// Gets an immutable reference to the underlying asynchronous HTTP client.
    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    /// Creates a client from login credentials.
    ///
    /// # Errors
    ///
    /// This function returns an error if the server responds with an invalid encoding or if the response does not contain a login token.
    /// Note that the absence of a login token isn't necessarily the server's fault; this error also occurs if the user specifies an invalid URL or an unsupported protocol.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```ignore
    /// #![feature(async_await)]
    ///
    /// use smartschool::Client;
    ///
    /// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
    /// assert_eq!("https://myschool.smartschool.be", client.url());
    /// ```
    ///
    /// # Implementation
    ///
    /// Logging into Smartschool follows a fairly complex process.
    ///
    /// [1] First, we have to make a GET request to the `/login` path.
    /// [2] We then have to save the `PHPSESSID` cookie contained in the response headers.
    /// [3] Next, we need to extract the login token from the HTML response's body.
    /// [4] We're almost done - we now have to send all our authentication data to `/login`.
    /// This means making a POST request containing username, password, token and the `PHPSESSID` cookie from earlier.
    /// [5] Finally, we have to get a new `PHPSESSID` cookie from the response headers.
    /// This cookie can then be used to authenticate oneself across the platform.
    /// If the headers don't contain a new `PHPSESSID`, the credentials are invalid or the token has expired, and we return an error.
    ///
    /// Since cookies are managed automatically by the HTTP client's cookie store, the `PHPSESSID` cookie doesn't get explicitly mentioned in the code.
    pub async fn login(url: &'a str, username: &'a str, password: &'a str) -> Result<Client<'a>> {
        let http_client = HttpClient::builder()
            .cookie_store(true)
            .redirect(RedirectPolicy::none())
            .build()?;

        // step 1 and step 2
        let request_url = format!("{}/login", url);
        let response = http_client
            .get(&request_url)
            .send()
            .compat()
            .await?
            .error_for_status()?;

        // step 3
        let body = response.into_body().compat().try_concat().await?;
        let body = std::str::from_utf8(&body)
            .map_err(|_| Error::Server("Server response was not UTF-8-encoded"))?;
        let token =
            get_token(body).ok_or(Error::Server("Server response did not contain login token"))?;

        // step 4 and step 5
        let mut form = HashMap::new();
        form.insert("login_form[_password]", password);
        form.insert("login_form[_token]", token);
        form.insert("login_form[_username]", username);
        let response = http_client
            .post(&request_url)
            .form(&form)
            .send()
            .compat()
            .await?
            .error_for_status()?;

        let successful = response
            .cookies()
            .any(|cookie| cookie.name() == "PHPSESSID");

        if successful {
            Ok(Client { http_client, url })
        } else {
            Err(Error::Client("Invalid login credentials"))
        }
    }

    /// Returns the URL of the associated Smartschool instance.
    pub fn url(&self) -> &str {
        self.url
    }
}

/// Extracts the login token from a response body.
fn get_token(body: &str) -> Option<&str> {
    // The token's <input> element happens to be the only one on the page that has two spaces before the `value` attribute.
    // If you get an error saying the token is missing, this trick probably doesn't work anymore.
    Regex::new("  value=\"(.+?)\"")
        .expect("error while creating smartschool::client::get_token regex")
        .captures(body)
        .and_then(|captures| captures.get(1))
        .map(|capture| capture.as_str())
}
