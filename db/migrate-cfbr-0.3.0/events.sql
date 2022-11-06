CREATE TYPE rr_event AS ENUM('notification', 'change_team');

CREATE TABLE event (
    id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    flavor rr_event, 
    time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    payload Json
);