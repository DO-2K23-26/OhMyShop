# Merger

The Merger acts as a Kafka consumer which will read messages from the source topics (for example: Client, Command, Product) to merge or transform the data according to specific rules, before publishing them in a new Kafka topic "Invoice".

## Invoice

The invoice will contain the following information:

```json
{
  "Timestamp": "2025-01-13T10:00:00Z", // Add a timestamp to trace the origin of the merged data
  "Client": {
    "id": "int",
    "name": "string",
    "email": "string",
    "address": "string"
  },
  "Command": {
    "id": "int",
    "date": "date",
    "Products": [
      {
        "id": "int",
        "name": "string",
        "price": "float"
      },
      {
        "id": "int",
        "name": "string",
        "price": "float"
      }
    ],
    "totalPrice": "float" // Calculate the total price of the command
  }
}
```

## Conditions

The Merger will merge the data from the source topics according to the following rules:
- If command is found in the Command topic, the Merger will look for the corresponding client in the Client topic.
- If I don't find the client or the products after 5 minutes, I will put the command in a dead letter queue.
- If the command is found, the Merger will calculate the total price of the command by summing the prices of the products and add it to the invoice to be published in the Invoice topic.