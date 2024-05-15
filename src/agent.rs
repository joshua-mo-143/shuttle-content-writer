use crate::errors::ApiError;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use async_openai::{config::OpenAIConfig, Client as OpenAIClient};
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::env;

pub trait Agent {
    fn name(&self) -> String;
    fn client(&self) -> OpenAIClient<OpenAIConfig>;
    fn system_message(&self) -> String;

    async fn prompt(&self, input: &str, data: String) -> Result<String, ApiError> {
        let input = format!(
            "{input}

            Provided context:
            {}
            ",
            serde_json::to_string_pretty(&data)?
        );
        let res = self
            .client()
            .chat()
            .create(
                CreateChatCompletionRequestArgs::default()
                    .model("gpt-4o")
                    .messages(vec![
                        //First we add the system message to define what the Agent does
                        ChatCompletionRequestMessage::System(
                            ChatCompletionRequestSystemMessageArgs::default()
                                .content(&self.system_message())
                                .build()?,
                        ),
                        //Then we add our prompt
                        ChatCompletionRequestMessage::User(
                            ChatCompletionRequestUserMessageArgs::default()
                                .content(input)
                                .build()?,
                        ),
                    ])
                    .build()?,
            )
            .await
            .map(|res| {
                //We extract the first one
                res.choices[0].message.content.clone().unwrap()
            })?;

        println!("Retrieved result from prompt: {res}");

        Ok(res)
    }
}

#[derive(Clone)]
pub struct Researcher {
    http_client: reqwest::Client,
    system: Option<String>,
    openai_client: OpenAIClient<OpenAIConfig>,
}

impl Researcher {
    pub fn new() -> Self {
        let api_key = env::var("OPENAI_API_KEY").unwrap();
        let config = OpenAIConfig::new().with_api_key(api_key);

        let openai_client = OpenAIClient::with_config(config);

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-API-KEY",
            env::var("SERPER_API_KEY").unwrap().parse().unwrap(),
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        Self {
            http_client,
            system: None,
            openai_client,
        }
    }

    pub async fn prepare_data(&self, prompt: &str) -> Result<String, ApiError> {
        let json = serde_json::json!({
            "q": prompt
        });

        let res = self
            .http_client
            .post("https://google.serper.dev/search")
            .json(&json)
            .send()
            .await
            .unwrap();

        let json = res.json::<Value>().await?;
        Ok(serde_json::to_string_pretty(&json)?)
    }
}

impl Agent for Researcher {
    fn name(&self) -> String {
        "Researcher".to_string()
    }

    fn client(&self) -> OpenAIClient<OpenAIConfig> {
        self.openai_client.clone()
    }

    fn system_message(&self) -> String {
        if let Some(message) = &self.system {
            message.to_owned()
        } else {
            "You are an agent.

        You will receive a question that may be quite short or does not have much context.
        Your job is to research the Internet and to return with a high-quality summary to the user, assisted by the provided context.
        The provided context will be in JSON format and contains data about the initial Google results for the website or query.

        Be concise.

        Question:
        ".to_string()
        }
    }
}

#[derive(Clone)]
pub struct Writer {
    system: Option<String>,
    client: OpenAIClient<OpenAIConfig>,
}

impl Writer {
    pub fn new() -> Self {
        let api_key = env::var("OPENAI_API_KEY").unwrap();
        let config = OpenAIConfig::new().with_api_key(api_key);

        let client = OpenAIClient::with_config(config);
        Self {
            system: None,
            client,
        }
    }
}

impl Agent for Writer {
    fn name(&self) -> String {
        "Writer".to_string()
    }

    fn client(&self) -> OpenAIClient<OpenAIConfig> {
        self.client.clone()
    }

    fn system_message(&self) -> String {
        if let Some(message) = &self.system {
            message.to_owned()
        } else {
            "You are an agent.

        You will receive some context from another agent about some Google results that a user has searched.
        Your job is to research the Internet and to write a high-quality article that a user has written. The article must not appear to be AI written. The article should be SEO optimised without overly compromising the
        quality of the article.

        You are free to be as creative as you wish. However, each paragraph must have the following:
        - The point you are trying to make
        - If there is a follow up action point
        - Why the follow up action point exists (or why the user needs to carry it out)

        Search query:
".to_string()
        }
    }
}
