# Merger

The Merger acts as a Kafka consumer which will read messages from the source topics (for example: Client, Command, Product) to merge or transform the data according to specific rules, before publishing them in a new Kafka topic "Invoice".

## Interface

Invoice
```json
{
  "id": "int",
  "date": "date",
  "client": {
    "id": "int",
    "name": "string",
    "email": "string",
    "address": "string"
  },
  "products": [
    {
      "id": "int",
      "name": "string",
      "price": "float",
      "command_id": "int"
    },
    {
      "id": "int",
      "name": "string",
      "price": "float",
      "command_id": "int"
    },
    ...
  ],
  "total_price": "float",
  "size": "int"
}
```

### Launch the producer
```bash
cargo run -p producer
```