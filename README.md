# OhMyShop

## Producer

The producer is a simple program that generates a random products, clients and command and sends them to the Kafka topic. More informations to be found in the producer's README.md.

### Launch the producer
```bash
# Launch the database & kafka
docker compose up -d
cargo run -p producer
```