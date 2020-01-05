//! A client for interacting with a Smartschool instance.

use crate::{
    error::{Error, Result},
    http::TrySend,
};
use regex::Regex;
use reqwest::{redirect, Client as HttpClient};
use std::collections::HashMap;

/// Extracts the request parameters from a response body.
fn get_params(body: &str) -> Option<(&str, &str)> {
    // The parameters' <input> elements happen to be the only ones on the page that
    // have two spaces before the `value` attribute. If you get a seemingly
    // unexplainable authentication error, this trick probably stopped working.
    let params: Vec<&str> = Regex::new("  value=\"(.+?)\"")
        .ok()?
        .captures_iter(body)
        .filter_map(|captures| captures.get(1))
        .map(|capture| capture.as_str())
        .collect();
    Some((params.get(0)?, params.get(1)?))
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
    /// The URL specifies the location of the Smartschool instance, usually a
    /// subdomain of `smartschool.be`.
    ///
    /// # Errors
    ///
    /// Returns an error in the following situations:
    ///
    /// * The login credentials are incorrect.
    /// * The URL is invalid or uses an unsupported protocol.
    /// * The server response didn't contain the login parameters.
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
        let (gen_time, token) = get_params(&response).ok_or(Error::Authentication)?;

        let mut form = HashMap::new();
        form.insert("login_form[_generationTime]", gen_time);
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
