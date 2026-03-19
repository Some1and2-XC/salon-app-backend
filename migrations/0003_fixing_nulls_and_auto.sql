PRAGMA foreign_keys = OFF;

-- task_categories -------------------------------------------------
CREATE TABLE task_categories_new (
    id   INTEGER PRIMARY KEY NOT NULL,
    name TEXT    NOT NULL
);
INSERT INTO task_categories_new SELECT * FROM task_categories;
DROP TABLE task_categories;
ALTER TABLE task_categories_new RENAME TO task_categories;

-- tasks -----------------------------------------------------------
CREATE TABLE tasks_new (
    id               INTEGER PRIMARY KEY NOT NULL,
    name             TEXT    NOT NULL,
    time_for_booking INTEGER NOT NULL,
    price_cad_cent   INTEGER,
    task_category_id INTEGER REFERENCES task_categories(id),
    date_created     INTEGER,
    last_modified    INTEGER
);
INSERT INTO tasks_new SELECT * FROM tasks;
DROP TABLE tasks;
ALTER TABLE tasks_new RENAME TO tasks;

-- appointment_states ----------------------------------------------
CREATE TABLE appointment_states_new (
    id   INTEGER PRIMARY KEY NOT NULL,
    name TEXT    NOT NULL
);
INSERT INTO appointment_states_new SELECT * FROM appointment_states;
DROP TABLE appointment_states;
ALTER TABLE appointment_states_new RENAME TO appointment_states;

-- appointment_availability ----------------------------------------
CREATE TABLE appointment_availability_new (
    id          INTEGER PRIMARY KEY NOT NULL,
    employee_id TEXT    REFERENCES employees(id),
    start_time  INTEGER NOT NULL,
    end_time    INTEGER NOT NULL,
    CONSTRAINT valid_time_range CHECK (end_time > start_time)
);
INSERT INTO appointment_availability_new SELECT * FROM appointment_availability;
DROP TABLE appointment_availability;
ALTER TABLE appointment_availability_new RENAME TO appointment_availability;

-- Fix indexes
CREATE INDEX IF NOT EXISTS idx_tasks_task_category_id ON tasks(task_category_id);

CREATE INDEX IF NOT EXISTS idx_availability_employee ON appointment_availability(employee_id);
CREATE INDEX IF NOT EXISTS idx_availability_start    ON appointment_availability(start_time);

PRAGMA foreign_keys = ON;
