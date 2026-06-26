# Monitor — Web App para visualización de pacientes OMNI

## Descripción General

Aplicación web independiente que **consume la misma base de datos** que `pdms-omni` (SQLite / Postgres / MSSQL) para visualizar pacientes, terapias e historial de telemetría. También administra las direcciones IP de las máquinas OMNI para redirección a sus interfaces web.

La BD ya está poblada por `pdms-omni`; el monitor solo añade `CREATE TABLE IF NOT EXISTS` para sus tablas propias.

---

## Stack

| Capa | Tecnología |
|------|-----------|
| Frontend | **Leptos** (SSR + WASM, Rust) |
| Backend | **Axum** (Rust, async) |
| ORM | **sqlx** (SQLite / Postgres) |
| Auth | JWT (HS256) + Argon2 — reutiliza `users` de la BD existente |
| Excel | `rust_xlsxwriter` |

---

## Tablas en la Base de Datos

### Tablas existentes de pdms-omni (solo lectura/consulta)

| Tabla | Uso |
|-------|-----|
| `users` | Autenticación (login, roles `admin`/`operator`/`viewer`) |
| `patients` | Listado de pacientes |
| `therapies` | Terapias por paciente, asociadas a máquina |
| `machines` | Máquinas OMNI registradas por serie + versión |
| `telemetry` | Lecturas de telemetría por terapia |
| `signals` | Definiciones de señales |
| `attribute_equivalences` | Mapeo valor numérico → nombre mostrable |
| `therapy_comments` | Comentarios de enfermería |
| `serial_sessions` | Sesiones de conexión serial |
| `session_readings` | Lecturas fuera de terapia |

### Nueva tabla: `machine_ips`

Asocia cada máquina OMNI (por FK a `machines.id`) a direcciones IP para redirección al web UI de la máquina. Una máquina puede tener varias IPs históricas, pero solo una activa (`is_active=1`).

#### SQLite
```sql
CREATE TABLE IF NOT EXISTS machine_ips (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    machine_id    INTEGER NOT NULL,
    ip_address    TEXT NOT NULL,
    port          INTEGER DEFAULT 9001,
    label         TEXT DEFAULT '',
    is_active     INTEGER DEFAULT 1,
    created_at    DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at    DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (machine_id) REFERENCES machines(id)
);
```

#### Postgres
```sql
CREATE TABLE IF NOT EXISTS machine_ips (
    id            BIGSERIAL PRIMARY KEY,
    machine_id    BIGINT NOT NULL REFERENCES machines(id),
    ip_address    TEXT NOT NULL,
    port          INTEGER DEFAULT 9001,
    label         TEXT DEFAULT '',
    is_active     BOOLEAN DEFAULT TRUE,
    created_at    TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

#### MSSQL
```sql
IF NOT EXISTS (SELECT * FROM sys.tables WHERE name = 'machine_ips')
CREATE TABLE machine_ips (
    id            INT IDENTITY(1,1) PRIMARY KEY,
    machine_id    INT NOT NULL,
    ip_address    NVARCHAR(200) NOT NULL,
    port          INT DEFAULT 9001,
    label         NVARCHAR(500) DEFAULT '',
    is_active     BIT DEFAULT 1,
    created_at    DATETIME2 DEFAULT GETUTCDATE(),
    updated_at    DATETIME2 DEFAULT GETUTCDATE(),
    FOREIGN KEY (machine_id) REFERENCES machines(id)
);
```

Índices para búsqueda rápida:
```sql
CREATE INDEX IF NOT EXISTS idx_machine_ips_machine ON machine_ips(machine_id);
CREATE INDEX IF NOT EXISTS idx_machine_ips_active ON machine_ips(machine_id, is_active);
```

---

## API REST

### Autenticación (tabla `users` existente)

| Método | Ruta | Auth | Descripción |
|--------|------|------|-------------|
| POST | /api/auth/login | No | `{username, password}` → `{token, user}` |
| GET | /api/auth/me | JWT | Perfil del usuario autenticado |

Reutiliza JWT + Argon2 de pdms-omni: `JwtClaims { user_id, username, full_name, email, role }`, HS256, issuer `pdms-omni` (mismo secreto para compatibilidad de tokens).

Roles: `admin`, `operator`, `viewer`.

### Admin — IPs de Máquinas (`machine_ips`)

| Método | Ruta | Auth | Rol | Body / Query | Descripción |
|--------|------|------|-----|-------------|-------------|
| GET | /api/admin/machine-ips | JWT | admin | — | Listar todas (JOIN con `machines` para mostrar `serial_number`) |
| POST | /api/admin/machine-ips | JWT | admin | `{machine_id, ip_address, port?, label?}` | Crear |
| PUT | /api/admin/machine-ips/{id} | JWT | admin | `{ip_address?, port?, label?, is_active?}` | Editar |
| DELETE | /api/admin/machine-ips/{id} | JWT | admin | — | Soft delete (`is_active=0`) |

### Admin — Usuarios (tabla `users`)

| Método | Ruta | Auth | Rol | Descripción |
|--------|------|------|-----|-------------|
| GET | /api/users | JWT | admin | Listar usuarios |
| POST | /api/users | JWT | admin | Crear usuario |
| PUT | /api/users/{id} | JWT | admin/self | Editar usuario |
| DELETE | /api/users/{id} | JWT | admin | Eliminar |

### Pacientes y Terapias (lectura)

| Método | Ruta | Auth | Descripción |
|--------|------|------|-------------|
| GET | /api/patients | JWT | Lista paginada de pacientes con última terapia |
| GET | /api/patients/{id} | JWT | Detalle del paciente (datos + terapias) |
| GET | /api/patients/{id}/therapies | JWT | Terapias del paciente con datos de máquina e IP |
| GET | /api/patients/{id}/history | JWT | Historial completo de telemetría (paginado) |
| GET | /api/patients/{id}/active-device | JWT | `{ip_address, port, url}` — IP de la máquina de la última/activa terapia |

### Dashboard (datos agregados)

| Método | Ruta | Auth | Descripción |
|--------|------|------|-------------|
| GET | /api/patients/{id}/dashboard | JWT | Promedios, máximos, mínimos por señal, timeline de fechas |
| GET | /api/therapies/{id}/dashboard | JWT | Dashboard para una terapia específica |

### Exportar

| Método | Ruta | Auth | Descripción |
|--------|------|------|-------------|
| GET | /api/patients/{id}/export | JWT | Descargar Excel del historial completo del paciente |
| GET | /api/therapies/{id}/export | JWT | Descargar Excel de terapia específica |

El Excel generado contiene:
- Hoja 1: datos crudos (Timestamp, Señal, Valor, Unidad, Equivalencia)
- Hoja 2: resumen estadístico por señal (promedio, min, max, count)
- Hoja 3 (opcional): gráfica embebida o tabla pivot por fecha

---

## Frontend (Leptos)

### Estructura de archivos

```
src/
├── app.rs                # Router principal con <Routes>
├── lib.rs                # Re-exportaciones
├── main.rs               # Entry point WASM/SSR
├── components/
│   ├── mod.rs
│   ├── login_form.rs     # Formulario login + manejo de JWT
│   ├── layout.rs         # Layout: sidebar + header + content
│   ├── patient_card.rs   # Tarjeta individual de paciente
│   ├── patient_table.rs  # Tabla responsiva de pacientes
│   ├── dashboard_chart.rs # Gráfica de líneas SVG
│   ├── stats_card.rs     # Tarjeta de estadística
│   └── machine_ip_form.rs # Formulario CRUD IP de máquina
├── pages/
│   ├── mod.rs
│   ├── login.rs              # /login
│   ├── patients.rs           # /patients — listado
│   ├── patient_detail.rs     # /patients/:id — detalle + opciones
│   ├── patient_history.rs    # /patients/:id/history — historial completo
│   ├── patient_dashboard.rs  # /patients/:id/dashboard
│   ├── therapy_detail.rs     # /therapies/:id — dashboard de terapia
│   ├── admin_machine_ips.rs  # /admin/machine-ips — CRUD
│   └── admin_users.rs        # /admin/users — CRUD
├── utils/
│   ├── api.rs            # Cliente HTTP (fetch con JWT)
│   ├── auth.rs           # Contexto de autenticación
│   └── excel.rs          # Trigger de descarga
```

### Rutas

| Ruta | Componente | Auth | Descripción |
|------|-----------|------|-------------|
| `/login` | `LoginPage` | No | Login |
| `/` | `PatientsPage` | JWT | Redirect a /patients |
| `/patients` | `PatientsPage` | JWT | Lista paginada de pacientes |
| `/patients/:id` | `PatientDetail` | JWT | Detalle: botones "Ver terapias" y "Ver historial" |
| `/patients/:id/history` | `PatientHistory` | JWT | Historial completo + filtros + export |
| `/patients/:id/dashboard` | `PatientDashboard` | JWT | Dashboard del paciente |
| `/therapies/:id` | `TherapyDetail` | JWT | Dashboard de terapia |
| `/admin/machine-ips` | `AdminMachineIps` | JWT+admin | CRUD de IPs de máquinas OMNI |
| `/admin/users` | `AdminUsers` | JWT+admin | CRUD de usuarios |

### Diseño UI — Glassmorphism

Toda la interfaz usa el estilo **Glassmorphism**:

- Fondos con `backdrop-filter: blur(12px)` y `background: rgba(255, 255, 255, 0.15)` (o variante oscura)
- Tarjetas, sidebar y modales con efecto de vidrio esmerilado
- Bordes sutiles con `border: 1px solid rgba(255, 255, 255, 0.2)`
- Sombras suaves: `box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1)`
- Paleta de gradientes fríos (azul marino → violeta) como fondo de página
- Texto blanco/oscuro con buen contraste sobre el vidrio
- Inputs, botones y tablas mantienen el mismo estilo glass con variante ligeramente más opaca

Implementado con clases CSS modulares en Leptos (archivo `src/styles/glassmorphism.css`).

### Flujo "Ver terapias" (redirección a IP de máquina OMNI)

1. Usuario hace clic en "Ver terapias" para un paciente
2. Frontend llama a `GET /api/patients/{id}/active-device`
3. Backend consulta la última terapia activa del paciente → obtiene `machine_id`
4. Cruza con `machine_ips` WHERE `machine_id = ? AND is_active = 1` → devuelve `{ ip_address, port, url: "http://{ip_address}:{port}", serial_number }`
5. Frontend redirige con `window.location.href = "http://{ip_address}:{port}"`
6. Si no hay IP registrada, muestra alerta: "No hay IP registrada para la máquina OMNI de este paciente"

### Dashboard de historial

- **Selector de señales** (checkboxes con los `signal_id` disponibles)
- **Selector de rango de fechas**
- **Gráfica de líneas** (SVG nativo con Leptos, sin librerías externas pesadas)
- **Tabla de datos** paginada
- **Estadísticas**: promedio, mínimo, máximo por señal en el rango seleccionado
- **Botón Exportar Excel** → descarga archivo `.xlsx`

---

## Configuración (`.env` del proyecto monitor)

```
# Base de datos (misma que pdms-omni)
DB_CONNECTION=sqlite
DB_DATABASE=../pdms-omni/database.db
# DB_HOST, DB_PORT, DB_USERNAME, DB_PASSWORD para postgres/mssql

# Servidor
MONITOR_HOST=127.0.0.1
MONITOR_PORT=9002

# Auth (misma secret que pdms-omni para compatibilidad de tokens)
JWT_SECRET=cambiar-este-secreto-en-produccion
JWT_EXPIRATION_HOURS=24

# Admin por defecto (solo aplica si no existe ningún usuario en la tabla users)
ADMIN_PASSWORD=cambiar-esta-contraseña

# CORS
CORS_ORIGINS=http://localhost:9002
```

Estructura `AppConfig` para el monitor:
```rust
pub struct MonitorConfig {
    pub db_connection: String,
    pub db_host: String,
    pub db_port: u16,
    pub db_database: String,
    pub db_username: String,
    pub db_password: String,
    pub monitor_host: String,
    pub monitor_port: u16,
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub admin_password: String,
    pub cors_origins: Vec<String>,
}
```

---

## Estructura del proyecto

```
C:\PROJECTS\Rust\monitor\
├── .env
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── config.rs          # MonitorConfig + from_env()
│   ├── database.rs        # Conexión + CREATE TABLE IF NOT EXISTS para omni_devices
│   ├── auth.rs            # JWT / Argon2 (copia de pdms-omni)
│   ├── models.rs          # Structs para serialización
│   ├── api/
│   │   ├── mod.rs
│   │   ├── auth.rs        # Handlers de login/me
│   │   ├── machine_ips.rs # CRUD machine_ips
│   │   ├── users.rs       # CRUD users
│   │   ├── patients.rs    # Endpoints de pacientes/terapias/historial
│   │   ├── dashboard.rs   # Datos agregados para dashboard
│   │   └── export.rs      # Generación de Excel
│   └── server.rs          # Axum router + montaje
├── static/                # Build output de Leptos
└── dashboard/             # (opcional) fuente Leptos si se desarrolla aparte
```

---

## Notas de Implementación

- El monitor **nunca** ejecuta migraciones sobre tablas existentes de pdms-omni. Solo `CREATE TABLE IF NOT EXISTS` para `machine_ips`.
- Los tokens JWT emitidos por pdms-omni y monitor son intercambiables si comparten `JWT_SECRET`.
- La tabla `users` tiene prioridad: si ya hay usuarios, no se crea el admin por defecto (misma lógica que pdms-omni).
- El puerto por defecto es `9002` para no chocar con pdms-omni (`9001`).
- Para el Excel, usar `rust_xlsxwriter` que es puro Rust y no necesita dependencias del SO.
- Las gráficas del dashboard se dibujan con SVG puro en Leptos (sin recharts ni librerías JS).
