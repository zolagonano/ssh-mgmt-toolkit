-- Your SQL goes here
CREATE TABLE sells (
  id SERIAL PRIMARY KEY,
  user_id BIGINT REFERENCES users(id),
  ref_id BIGINT,
  service_id INTEGER REFERENCES services(id),
  node_id INTEGER REFERENCES nodes(id),
  firstbuy_date TIMESTAMP,
  invoice_date TIMESTAMP,
  username TEXT,
  password TEXT,
  password_hash TEXT,
  status INTEGER NOT NULL,
)
