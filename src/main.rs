use std::env;
use regex::Regex;
use serde_json::json;
use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::Ready,
    },
    prelude::*
};

use google_cloud_pubsub::client::{Client as PubSubClient, ClientConfig as PubSubClientConfig};
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_gax::grpc::Status;
use tokio::task::JoinHandle;

struct  Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let article_pattern = Regex::new(r"^http").unwrap();
        if article_pattern.is_match(&msg.content) {
            let data = json!({
                "article_url": msg.content
            });
            let pubsub_message = serde_json::to_vec(&data).unwrap();

            let pubsub_config  = PubSubClientConfig::default().with_auth().await.unwrap();

            // Create pubsub client.
            let client = PubSubClient::new(pubsub_config).await.unwrap();

            // Create topic.
            let pubsub_topic = env::var("PUBSUB_TOPIC").expect("Expected a token in the environment");
            let topic = client.topic(pubsub_topic.as_str());
            if !topic.exists(None).await.unwrap() {
                topic.create(None, None).await.unwrap();
            }

            // Start publisher.
            let publisher = topic.new_publisher(None);

            // Publish message.
            let tasks : Vec<JoinHandle<Result<String,Status>>> = (0..1).into_iter().map(|_i| {
                let publisher = publisher.clone();
                let pubsub_message = pubsub_message.clone();
                tokio::spawn(async move {
                    let msg = PubsubMessage {
                    data: pubsub_message,
                    // Set ordering_key if needed (https://cloud.google.com/pubsub/docs/ordering)
                    ordering_key: "order".into(),
                    ..Default::default()
                    };

                    // Send a message. There are also `publish_bulk` and `publish_immediately` methods.
                    let awaiter = publisher.publish(msg).await;

                    // The get method blocks until a server-generated ID or an error is returned for the published message.
                    awaiter.get().await
                })
            }).collect();

            // Wait for all publish task finish
            for task in tasks {
                let _message_id = task.await.unwrap();
            }

            // Wait for publishers in topic finish.
            let mut publisher = publisher;
            publisher.shutdown().await;

            if let Err(why) = msg.channel_id.say(&ctx.http, "OK").await {
                println!("Error sending message: {why:?}");
            }

        }
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}



#[tokio::main]
async fn main() {
    let token = env::var("DISCROD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES 
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("client error: {why:?}");
    }
}