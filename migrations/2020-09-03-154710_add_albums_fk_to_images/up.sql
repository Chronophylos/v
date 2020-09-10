ALTER TABLE images
    ADD CONSTRAINT album_fk
        FOREIGN KEY (album_id)
        REFERENCES albums(id)
        ON DELETE CASCADE