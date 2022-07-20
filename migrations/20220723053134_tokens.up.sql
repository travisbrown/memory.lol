CREATE TABLE github_tokens(
    value VARCHAR(255) NOT NULL PRIMARY KEY,
    id UNSIGNED BIGINT NOT NULL,
    gist BOOLEAN NOT NULL,
    FOREIGN KEY (id) REFERENCES github_names (id)
);

CREATE TABLE google_tokens(
    value VARCHAR(255) NOT NULL PRIMARY KEY,
    id VARCHAR(255) NOT NULL,
    FOREIGN KEY (id) REFERENCES google_names (id)
);

CREATE TABLE twitter_tokens(
    value VARCHAR(255) NOT NULL PRIMARY KEY,
    id UNSIGNED BIGINT NOT NULL,
    consumer_secret VARCHAR(255) NOT NULL,
    access_key VARCHAR(255) NOT NULL,
    access_secret VARCHAR(255) NOT NULL,
    FOREIGN KEY (id) REFERENCES twitter_names (id)
);
