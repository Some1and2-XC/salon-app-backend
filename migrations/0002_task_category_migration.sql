-- Adds primary key to table.
CREATE TABLE task_categories_new (
    id      BIGINT     PRIMARY KEY NOT NULL,
    name    TEXT       NOT NULL
);

-- Puts all the data from task_categories into the new table
INSERT INTO task_categories_new (id, name)
SELECT id, name FROM task_categories;

-- Deletes old table
DROP TABLE task_categories;

-- Renames the table
ALTER TABLE task_categories_new RENAME TO task_categories;
