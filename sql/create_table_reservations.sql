CREATE TABLE reservations (
  id                SERIAL PRIMARY KEY
  , user_id         SERIAL
  , resource_id     SERIAL
  , start_datetime  TIMESTAMP WITH TIME ZONE NOT NULL
  , end_datetime    TIMESTAMP WITH TIME ZONE NOT NULL
  , description     TEXT
  , passhash        CHAR(64)
);