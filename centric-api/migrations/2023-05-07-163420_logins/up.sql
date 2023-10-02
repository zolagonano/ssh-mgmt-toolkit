-- Your SQL goes here

CREATE TABLE logins(
    id SERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    admin BOOLEAN NOT NULL DEFAULT FALSE,
    register_date TIMESTAMP NOT NULL DEFAULT NOW()
)
