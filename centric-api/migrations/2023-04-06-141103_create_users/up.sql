-- Your SQL goes here
CREATE TABLE users (
  id BIGINT PRIMARY KEY,
  ref_id BIGINT,
  register_date TIMESTAMP NOT NULL DEFAULT NOW()
)
