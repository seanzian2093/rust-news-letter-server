# rust-news-letter-server

A repo for a news letter server

## Install `sqlx-cli`

`cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres`
`cargo install sqlx-cli --no-default-features --features postgres`

## Run `init_db.sh`

`SKIP_DOCKER=true ./scripts/init_db.sh`

## curl

`-v` for verbose output
`curl http://127.0.0.1:3000/health_check -v`
`curl http://127.0.0.1:3000 -v`
`curl -X POST -H "Content-Type: application/json" -d '{"name": "seanz", "email": "seanz@seanz.com"}' http://127.0.0.1:3000/subscriptions`

## Prepare sqlx meta data - offline mode

`cargo sqlx prepare -- --bin rust-news-letter-server`

## Build Docker Image

`docker build --tag rust-news-letter-server --file Dockerfile .`
