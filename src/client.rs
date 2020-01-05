//! A client for interacting with a Smartschool instance.

use crate::{
    error::{Error, Result},
    http::TrySend,
};
use regex::Regex;
use reqwest::{redirect, Client as HttpClient};
use std::collections::HashMap;

/// Extracts the login token from a response body.
fn get_token(body: &str) -> Option<&str> {
    // The token's <input> element happens to be the only one on the page that has
    // two spaces before the `value` attribute. If you get an error saying the
    // token is missing, this trick probably doesn't work anymore.
    Regex::new("  value=\"(.+?)\"")
        .expect("error while compiling smartschool::client::get_token regex")
        .captures(body)
        .and_then(|captures| captures.get(1))
        .map(|capture| capture.as_str())
}

/// An asynchronous client for interacting with a Smartschool instance.
#[derive(Clone, Debug)]
pub struct Client<'a> {
    http_client: HttpClient,
    url: &'a str,
}

impl<'a> Client<'a> {
    /// Logs in with the provided login credentials and returns a client.
    ///
    /// # Errors
    ///
    /// Returns an error in the following situations:
    ///
    /// * The URL is invalid or uses an unsupported protocol.
    /// * The server response doesn't contain a login token.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```ignore
    /// use smartschool::Client;
    ///
    /// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
    ///
    /// assert_eq!("https://myschool.smartschool.be", client.url());
    /// ```
    pub async fn login(url: &'a str, username: &str, password: &str) -> Result<Client<'a>> {
        let http_client = HttpClient::builder()
            .cookie_store(true)
            .redirect(redirect::Policy::none())
            .build()?;

        let request_url = format!("{}/login", url);
        let response = http_client
            .get(&request_url)
            .try_send()
            .await?
            .text()
            .await?;
        let token = get_token(&response).ok_or(Error::Authentication)?;

        let mut form = HashMap::new();
        form.insert("login_form[_password]", password);
        form.insert("login_form[_token]", token);
        form.insert("login_form[_username]", username);
        let response = http_client
            .post(&request_url)
            .form(&form)
            .try_send()
            .await?;

        let successful = response
            .cookies()
            .any(|cookie| cookie.name() == "PHPSESSID");

        if successful {
            Ok(Client { http_client, url })
        } else {
            Err(Error::Authentication)
        }
    }

    /// Gets an immutable reference to the underlying asynchronous HTTP client.
    pub(crate) fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    /// Returns the URL of the associated Smartschool instance.
    pub fn url(&self) -> &str {
        self.url
    }
}
