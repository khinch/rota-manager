CREATE TABLE shifts (
    id UUID PRIMARY KEY,
    member_id UUID NOT NULL,
    day SMALLINT NOT NULL CHECK (day >= 0 AND day <= 6),
    in_time SMALLINT NOT NULL CHECK (in_time >= 0 AND in_time <= 1440),
    out_time SMALLINT NOT NULL CHECK (out_time >= 0 AND out_time <= 1440)
);
