-- ============================================================
-- Initial schema
-- All timestamps are stored as Unix milliseconds (BIGINT).
-- ============================================================

-- Users -----------------------------------------------------------
CREATE TABLE IF NOT EXISTS users (
    uuid          TEXT        PRIMARY KEY NOT NULL,
    -- uuid          UUID        PRIMARY KEY,
    phone         TEXT,
    email         TEXT        NOT NULL UNIQUE,
    first_name    TEXT        NOT NULL,
    last_name     TEXT        NOT NULL,
    date_created  BIGINT,
    last_modified BIGINT,
    admin         BOOLEAN     NOT NULL DEFAULT FALSE
);

-- Tasks -----------------------------------------------------------
CREATE TABLE IF NOT EXISTS tasks (
    id               BIGINT      PRIMARY KEY NOT NULL,
    name             TEXT        NOT NULL,
    time_for_booking BIGINT      NOT NULL,   -- seconds
    date_created     BIGINT,
    last_modified    BIGINT
);

-- Employees -------------------------------------------------------
CREATE TABLE IF NOT EXISTS employees (
    id            TEXT        PRIMARY KEY NOT NULL,
    first_name    TEXT        NOT NULL,
    last_name     TEXT        NOT NULL,
    phone         TEXT        NOT NULL,
    email         TEXT        NOT NULL UNIQUE,
    date_created  BIGINT,
    last_modified BIGINT
);

-- Appointment states (seeded) ------------------------------------
CREATE TABLE IF NOT EXISTS appointment_states (
    id   BIGINT  PRIMARY KEY,
    name TEXT    NOT NULL
);

INSERT INTO appointment_states (id, name) VALUES
    (0, 'Unconfirmed'),
    (1, 'Accepted'),
    (2, 'Confirmed'),
    (3, 'Cancelled'),
    (4, 'Completed')
ON CONFLICT DO NOTHING;

-- Appointments ----------------------------------------------------
CREATE TABLE IF NOT EXISTS appointments (
    uuid                  TEXT    PRIMARY KEY,
    user_uuid             TEXT    NOT NULL,
    task_id               BIGINT  NOT NULL REFERENCES tasks(id),
    employee_id           TEXT    REFERENCES employees(id),
    start_time            BIGINT  NOT NULL,
    length                BIGINT  NOT NULL,
    appointment_state_id  BIGINT  NOT NULL REFERENCES appointment_states(id) DEFAULT 0,
    date_created          BIGINT,
    last_modified         BIGINT,

    -- Mirror of frontend validate(): employee_id must be set for states 1, 2, 4
    CONSTRAINT employee_required_for_active_states CHECK (
        appointment_state_id NOT IN (1, 2, 4) OR employee_id IS NOT NULL
    )
);

CREATE INDEX IF NOT EXISTS idx_appointments_user_uuid ON appointments(user_uuid);
CREATE INDEX IF NOT EXISTS idx_appointments_employee_id ON appointments(employee_id);
CREATE INDEX IF NOT EXISTS idx_appointments_state ON appointments(appointment_state_id);
CREATE INDEX IF NOT EXISTS idx_appointments_start_time ON appointments(start_time);

-- Appointment availability ----------------------------------------
CREATE TABLE IF NOT EXISTS appointment_availability (
    id          BIGINT  PRIMARY KEY NOT NULL,
    employee_id TEXT    REFERENCES employees(id),
    start_time  BIGINT  NOT NULL,
    end_time    BIGINT  NOT NULL,

    CONSTRAINT valid_time_range CHECK (end_time > start_time)
);

CREATE INDEX IF NOT EXISTS idx_availability_employee ON appointment_availability(employee_id);
CREATE INDEX IF NOT EXISTS idx_availability_start ON appointment_availability(start_time);
