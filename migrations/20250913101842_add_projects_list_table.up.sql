CREATE TABLE projects_list (
    project_id UUID NOT NULL PRIMARY KEY,
    user_id UUID NOT NULL,
    project_name VARCHAR(255) NOT NULL
);
