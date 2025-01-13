CREATE TABLE Client (
  id INT PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  email VARCHAR(255) NOT NULL,
  address VARCHAR(255) NOT NULL
);

CREATE TABLE Product (
  id INT PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  price FLOAT NOT NULL
);

CREATE TABLE Command (
  id INT PRIMARY KEY,
  clientId INT NOT NULL,
  date DATE NOT NULL,
  FOREIGN KEY (clientId) REFERENCES Client(id)
);

CREATE TABLE CommandProduct (
  commandId INT NOT NULL,
  productId INT NOT NULL,
  PRIMARY KEY (commandId, productId),
  FOREIGN KEY (commandId) REFERENCES Command(id),
  FOREIGN KEY (productId) REFERENCES Product(id)
);