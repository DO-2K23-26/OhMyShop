
## Producer

A producer is a service that provides data to a consumer. In this case, the producer is a service that provides data about clients, orders and products to a consumer.

Here is a mermaid diagram of the producer's data model:

```mermaid
erDiagram
    CLIENT ||..o{ COMMAND : places
    COMMAND ||..o{ PRODUCT : contains
```

1. Client:
    ```json
    {
      "id": "int",
      "name": "string",
      "email": "string",
      "address": "string"
    }
    ```

2. Command:
    ```json
    {
        "id": "int",
        "clientId": "id",
        "date": "date",
        "produits": "List<Product>"
    }
    ```

3. Product:
    ```json
    {
        "id": "int",
        "name": "string",
        "price": "float"
    }
    ```

### Initial setup of database
```bash
docker compose up -d
cargo install sqlx-cli
export DATABASE_URL=postgres://postgres:postgres@localhost/postgres
cargo sqlx migrate run
cargo run -p producer -- --seed
```

### Launch the producer
```bash
cargo run -p producer
```
