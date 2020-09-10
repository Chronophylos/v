CREATE TABLE albums (
    id SERIAL PRIMARY KEY,

    token VARCHAR(8) UNIQUE,
    deletion_token VARCHAR(16) UNIQUE,

    title VARCHAR(64)
)