#[cfg(test)]
mod test
{
    extern crate test;
    use std::{sync::Arc, time::Duration};

    use crate::*;
    use reqwest::{self,
                  cookie::{Cookie, Jar},
                  header::{CONTENT_TYPE, SET_COOKIE},
                  Client, RequestBuilder, Response};
    use serde_json::to_string as to_json;
    use test::Bencher;
    use tokio::test;

    const SERVER_URL: &'static str = "http://127.0.0.1:3334";

    fn make_client() -> Client
    {
        let client =
            reqwest::ClientBuilder::new().cookie_store(true)
                                         .build()
                                         .unwrap();
        //specify common headers, test different http modes, etc.
        return client;
    }

    enum Method
    {
        GET,
        POST,
        PUT,
        DELETE,
    }
    fn client_method(c: &Client, m: Method, url: &str) -> RequestBuilder
    {
        match m
        {
            Method::GET => c.get(url),
            Method::POST => c.post(url),
            Method::PUT => c.put(url),
            Method::DELETE => c.delete(url),
        }
    }

    async fn api_request<T>(c: &Client, route: &str, m: Method, body: &T) -> Result<Response>
        where T: ?Sized + Serialize
    {
        let req = client_method(c,m,& (format!("{}/api/{}", SERVER_URL, route)))
                    .header(CONTENT_TYPE, "application/json")
                    .body(to_json(body).unwrap());
        let res = req.send().await?; 
        return Ok(res);
    }

    #[test]
    async fn get_session() -> Result<()>
    {
        let client = make_client();
        let res1 = api_request(&client,
                               "get_session",
                               Method::POST,
                               &SessionRequest { answer:    "please".to_string(),
                                                 challenge: "hi".to_string(), }).await?;
        assert!(res1.headers()
                    .get(SET_COOKIE)
                    .expect("no set cookie header was set")
                    .to_str()?
                    .contains("session_id"));
        let (key, value) = ("hi".to_string(), "bye".to_string());
        let _res2 = api_request(&client,
                                "kv_set",
                                Method::PUT,
                                &KVSetRequest { key: key.clone(),
                                                value }).await?;
        let res3 = api_request(&client, "kv_get", Method::POST, &KVGetRequest { key }).await?
                                                                                      .text()
                                                                                      .await?;
        let kvrow = serde_json::from_str::<KVRow>(&res3).unwrap();
        assert!(kvrow.value == "bye");
        return Ok(());
    }

    #[test]
    async fn db_clear() -> Result<()>
    {
        let client = make_client();
        let (key, value) = ("hi".to_string(), "bye".to_string());
        let _res1 = api_request(&client,
                                "kv_set",
                                Method::PUT,
                                &KVSetRequest { key: key.clone(),
                                                value }).await?;
        let res2 = api_request(&client, "kv_get", Method::POST, &KVGetRequest { key:key.clone() }).await?
                                                                                      .text()
                                                                                      .await?;
        tokio::time::sleep(Duration::from_millis(5000)).await;
        let res3 = api_request(&client, "kv_get", Method::POST, &KVGetRequest { key }).await?
                                                                                      .text()
                                                                                      .await?;
        let kvrow = serde_json::from_str::<Option<KVRow>>(&res2)?;
        assert!(kvrow.is_some());                                                                     
        let kvrow = serde_json::from_str::<Option<KVRow>>(&res3)?;
        assert!(kvrow.is_none());
        return Ok(());
    }
}
