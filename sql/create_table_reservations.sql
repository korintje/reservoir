CREATE TABLE reservations (
  id                SERIAL PRIMARY KEY
  , member_id       SERIAL
  , machine_id      SERIAL
  , start_datetime  TIMESTAMP WITH TIME ZONE NOT NULL
  , end_datetime    TIMESTAMP WITH TIME ZONE NOT NULL
  , description     TEXT
  , passhash        CHAR(64)
);