CREATE TABLE images (
    id SERIAL PRIMARY KEY,
    album_id INT NOT NULL,

    token VARCHAR(8) UNIQUE,
    deletion_token VARCHAR(16) UNIQUE,

    url VARCHAR(255)
)