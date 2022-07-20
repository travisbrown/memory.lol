CREATE TABLE github_names(
    id UNSIGNED BIGINT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE google_names(
    id VARCHAR(255) NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE INDEX google_names_value ON google_names (value);

CREATE TABLE twitter_names(
    id UNSIGNED BIGINT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
);