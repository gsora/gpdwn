use std::fmt::Display;

use reqwest::Url;

const BASE_DOWNLOAD_URL: &str = "https://api.gopro.com/media/x/zip/source";

pub enum GPError {
    ReqwestError(reqwest::Error),
    URLError(url::ParseError),
    APIError(Vec<String>),
}

impl Display for GPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GPError::URLError(e) => write!(f, "url parsing error: {}", e),
            GPError::ReqwestError(e) => write!(f, "reqwest error: {}", e),
            GPError::APIError(e) => write!(f, "api errors: {}", e.join(",")),
        }
    }
}

impl From<url::ParseError> for GPError {
    fn from(value: url::ParseError) -> Self {
        Self::URLError(value)
    }
}

impl From<reqwest::Error> for GPError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Pages {
    current_page: i32,
    per_page: i32,
    total_items: i32,
    total_pages: i32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Media {
    id: String,
    gopro_media: bool,
    filename: String,

    #[serde(rename = "type")]
    typ: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Embedded {
    errors: Vec<String>,
    media: Vec<Media>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MediaResponse {
    #[serde(rename = "_pages")]
    pages: Pages,

    #[serde(rename = "_embedded")]
    embedded: Embedded,
}

#[derive(Default)]
struct MediaIDs {
    ids: Vec<String>,
    total_pages: i32,
}

pub struct Handle<'a> {
    auth_jwt: &'a str,
}

impl<'a> Handle<'a> {
    /// Returns a new instance of Handle.
    pub fn new(auth_jwt: &'a str) -> Self {
        Self {
            auth_jwt: auth_jwt.strip_prefix("Bearer ").unwrap_or(&auth_jwt),
        }
    }

    /// Builds a downloadable URL for the given ids, and JWT authentication token.
    pub fn download_url(&self, ids: Vec<String>) -> Result<String, GPError> {
        let mut u = Url::parse(BASE_DOWNLOAD_URL)?;

        u.query_pairs_mut()
            .append_pair("ids", &ids.join(","))
            .append_pair("access_token", &self.auth_jwt)
            .finish();

        Ok(u.to_string())
    }

    /// Returns all the media IDs found at the specified page, given the settings.
    fn media_ids_at_page(
        &self,
        page: i32,
        everything: bool,
        video_only: bool,
    ) -> Result<MediaIDs, GPError> {
        let c = reqwest::blocking::Client::new();

        let page_url = Handle::api_url(page);

        log::debug!("getting page {}", page_url);

        let request = c.get(page_url)
            .header("User-Agent", "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/114.0")
            .header("Accept", "application/vnd.gopro.jk.media+json; version=2.0.0")
            .header("Referer", "https://plus.gopro.com/")
            .bearer_auth(self.auth_jwt)
            .build()?;

        log::debug!("request: {:?}", request);

        let response: String = c.execute(request)?.text()?;

        // using serde manually to not lose error context
        let response: MediaResponse = serde_json::de::from_str(&response).unwrap();

        if !response.embedded.errors.is_empty() {
            return Err(GPError::APIError(response.embedded.errors));
        }

        let ret = MediaIDs {
            total_pages: response.pages.total_pages,
            ids: response
                .embedded
                .media
                .into_iter()
                .filter_map(|media_id| {
                    match (!media_id.gopro_media && !everything)
                        || (media_id.typ.to_lowercase() != "video" && video_only)
                    {
                        true => None,
                        false => Some(media_id.id),
                    }
                })
                .collect(),
        };

        Ok(ret)
    }

    /// Returns all the media ids available for the account associated with the authentication JWT.
    pub fn media_ids(&self, everything: bool, video_only: bool) -> Result<Vec<String>, GPError> {
        let mut counter = 1;

        let mut ret = vec![];

        loop {
            log::debug!("fetching media ids at page {}", counter);
            let mut ids = self.media_ids_at_page(counter, everything, video_only)?;
            ret.append(&mut ids.ids);
            counter += 1;
            if counter > ids.total_pages {
                break;
            }
        }

        Ok(ret)
    }

    fn api_url(page: i32) -> String {
        let mut u = Url::parse("https://api.gopro.com/").unwrap();
        u.set_path("media/search");

        u.query_pairs_mut()
            .append_pair("fields", "id,gopro_media,filename,type")
            .append_pair("order_by", "captured_at")
            .append_pair("per_page", "100")
            .append_pair("page", &format!("{page}"))
            .finish();

        u.to_string()
    }
}
