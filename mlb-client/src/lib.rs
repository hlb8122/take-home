pub mod types;

use reqwest::{Client, Error};
pub use time::Date;

use types::*;

const MLB_SCHEDULE_URL: &str = "http://statsapi.mlb.com/api/v1/schedule";
const HYDRATE_ARGS: &str = "game(content(editorial(recap))),decisions";
const DATE_FORMAT: &str = "%Y-%m-%d";

/// Client providing HTTP requests to the mlb API.
#[derive(Default)]
pub struct MlbClient {
    client: Client,
}

impl MlbClient {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn get_schedule_via_date(&self, date: &Date) -> Result<Schedule, Error> {
        let date_str = date.format(DATE_FORMAT);
        let query_params = [
            ("hydrate", HYDRATE_ARGS),
            ("date", &date_str),
            ("sportId", "1"),
        ];
        self.client
            .get(MLB_SCHEDULE_URL)
            .query(&query_params)
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_schedule_today(&self) -> Result<Schedule, Error> {
        // TODO: Double check timezones
        self.get_schedule_via_date(&Date::today()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn santity_fetch_schedule() {
        let client = MlbClient::new();
        let today = time::date!(2018 - 06 - 10);
        let schedule = client.get_schedule_via_date(&today).await;

        assert!(schedule.is_ok())
    }

    #[tokio::test]
    async fn santity_fetch_schedule_today() {
        let client = MlbClient::new();
        let schedule_today = client.get_schedule_today().await.unwrap();

        let today = Date::today();
        let schedule = client.get_schedule_via_date(&today).await.unwrap();

        assert_eq!(schedule, schedule_today)
    }
}
