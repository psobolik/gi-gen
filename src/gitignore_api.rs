/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-10
 */

use crate::gitignore_api::error::Error;

mod error;

const API_URL: &str = "https://www.toptal.com/developers/gitignore/api";

pub(super) async fn get_template_names() -> Result<Vec<String>, Error> {
    let url = format!("{API_URL}/list");
    let client = reqwest::Client::new();
    let request_builder = client.get(url);
    let response = request_builder.send().await?;
    let response = response.error_for_status()?;
    let response = response.text().await?;
    let mut vec = Vec::new();
    for lines in response.split('\n') {
        for template in lines.split(',') {
            vec.push(template.to_string());
        }
    }
    Ok(vec)
}

pub(super) async fn get_template(template_names: &[String]) -> Result<String, Error> {
    let url = format!("{API_URL}/{}", template_names.join(","));
    let client = reqwest::Client::new();
    let request_builder = client.get(url);
    let response = request_builder.send().await?;
    let response = response.error_for_status()?;
    Ok(response.text().await?)
}
