-- Your SQL goes here

CREATE TABLE nodes (
  id SERIAL PRIMARY KEY,
  address TEXT UNIQUE NOT NULL,
  token TEXT NOT NULL,
  status INTEGER NOT NULL DEFAULT 1
)
