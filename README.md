# Tobe Read Bot

a Discord bot designed to capture URLs of articles shared in a Discord channel, publishing the captured URL to a Pub/Sub topics.

## Run
```
export PUBSUB_TOPIC=''
export DISCORD_TOKEN=''
export GOOGLE_APPLICATION_CREDENTIALS=credentials.json
cargo run
```