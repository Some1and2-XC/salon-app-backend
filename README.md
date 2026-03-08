# appointment-backend

Rust/Axum backend for the appointment booking system, with Firebase Authentication.

## Stack

| Layer | Technology |
|---|---|
| Web framework | [Axum](https://github.com/tokio-rs/axum) 0.7 |
| Async runtime | Tokio |
| Database | PostgreSQL via [sqlx](https://github.com/launchbadge/sqlx) |
| Auth | Firebase ID tokens (RS256 JWT) |
| Migrations | sqlx built-in migrate! |

---

## Getting started

### 1. Prerequisites

- Rust (stable) — `rustup update stable`
- PostgreSQL running locally (or a connection string to a remote instance)
- A Firebase project with Authentication enabled

### 2. Environment

```bash
cp .env.example .env
# Edit .env — fill in DATABASE_URL and FIREBASE_PROJECT_ID
```

### 3. Run

```bash
cargo run
```

The server starts on `http://0.0.0.0:3000` (override with `PORT=`).  
Migrations run automatically on startup.

---

## Authentication

Every request (except `GET /tasks` and `GET /availability`) requires a Firebase ID token
in the `Authorization` header:

```
Authorization: Bearer <firebase-id-token>
```

The token is verified against Google's public keys for your project.  
Admin-only endpoints additionally require `admin = true` on the user's DB row.

---

## API reference

### Users
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/users` | 🔑 | Create your own user record (ties Firebase UID to a profile) |
| `GET` | `/users/me` | 🔑 | Get your own profile |
| `PATCH` | `/users/me` | 🔑 | Update your own profile |
| `GET` | `/users/:uuid` | 🔑 Admin | Get any user |

### Tasks
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/tasks` | Public | List all tasks |
| `GET` | `/tasks/:id` | Public | Get a single task |
| `POST` | `/tasks` | 🔑 Admin | Create a task |
| `PATCH` | `/tasks/:id` | 🔑 Admin | Update a task |
| `DELETE` | `/tasks/:id` | 🔑 Admin | Delete a task |

### Employees
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/employees` | 🔑 | List employees |
| `GET` | `/employees/:id` | 🔑 | Get a single employee |
| `POST` | `/employees` | 🔑 Admin | Create an employee |
| `PATCH` | `/employees/:id` | 🔑 Admin | Update an employee |
| `DELETE` | `/employees/:id` | 🔑 Admin | Delete an employee |

### Appointments
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/appointments` | 🔑 | List (own, or all if admin). Supports `?user_uuid`, `?employee_id`, `?state_id`, `?from`, `?to` |
| `POST` | `/appointments` | 🔑 | Create an appointment |
| `GET` | `/appointments/:uuid` | 🔑 | Get one (own or any if admin) |
| `PATCH` | `/appointments/:uuid` | 🔑 | Update (own or any if admin) |
| `DELETE` | `/appointments/:uuid` | 🔑 | Delete (own or any if admin) |

### Availability
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/availability` | 🔑 | Query slots. Supports `?employee_id`, `?from`, `?to` |
| `POST` | `/availability` | 🔑 Admin | Create a slot |
| `DELETE` | `/availability/:id` | 🔑 Admin | Delete a slot |

---

## Data model notes

- All timestamps are **Unix milliseconds** (`BIGINT`), matching the frontend.
- `Phone` numbers are stored as plain `TEXT`; validation (7–15 digits) happens at the API boundary.
- The `Appointment` validation rule from the frontend is enforced both in Rust code **and** as a PostgreSQL `CHECK` constraint.
- `AppointmentState` rows are seeded by the migration and should not be deleted.
