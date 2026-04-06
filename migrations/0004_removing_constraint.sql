-- Creates new table (without constraint in it)
CREATE TABLE IF NOT EXISTS appointments_new (
    uuid                  TEXT    PRIMARY KEY,
    user_uuid             TEXT    NOT NULL,
    task_id               BIGINT  NOT NULL REFERENCES tasks(id),
    employee_id           TEXT    REFERENCES employees(id),
    start_time            BIGINT  NOT NULL,
    length                BIGINT  NOT NULL,
    appointment_state_id  BIGINT  NOT NULL REFERENCES appointment_states(id) DEFAULT 0,
    date_created          BIGINT,
    last_modified         BIGINT
);

-- Copy and Rename and Remove
INSERT INTO appointments_new SELECT uuid, user_uuid, task_id, employee_id, start_time, length, appointment_state_id, date_created, last_modified FROM appointments;
DROP TABLE appointments;
ALTER TABLE appointments_new RENAME TO appointments;
