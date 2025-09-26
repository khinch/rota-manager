CREATE TABLE members (
    member_id UUID NOT NULL PRIMARY KEY,
    project_id UUID NOT NULL,
    member_name VARCHAR(255) NOT NULL
);
