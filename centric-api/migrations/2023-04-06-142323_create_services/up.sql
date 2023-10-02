-- Your SQL goes here

CREATE TABLE services (
  id SERIAL PRIMARY KEY,
  max_logins INTEGER NOT NULL,
  max_traffic INTEGER,
  price INTEGER NOT NULL,
  available BOOLEAN NOT NULL DEFAULT FALSE
)
