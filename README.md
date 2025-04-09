
# 📋 API de Gestión de Tareas

Una API RESTful + WebSocket desarrollada con **Rust**, **Rocket** y **RethinkDB** para gestionar tareas de usuarios.

---

## 🌐 URL Base

- API: `http://localhost:8000/v1/api/`
- Documentación Swagger UI: [http://localhost:8000/swagger-ui/](http://localhost:8000/swagger-ui/)

---

## 📦 Funcionalidades

- Crear un nuevo usuario
- Autenticar usuario (login con JWT)
- Crear nuevas tareas
- Obtener todas las tareas de un usuario autenticado
- Obtener tarea por ID
- Cambiar el estado de una tarea
- Eliminar una tarea por ID
- Eliminar todas las tareas de un usuario
- WebSocket para recibir en tiempo real los cambios en tareas

---

## 🔐 Autenticación

La mayoría de los endpoints requieren un token JWT en el encabezado:

```http
Authorization: Bearer <tu_token_jwt>
```

---

## 🔁 WebSocket

Conexión WebSocket para recibir cambios en tiempo real:

```
GET /v1/api/ws/<token>
```

Este WebSocket te permite recibir eventos cuando una tarea cambia o es creada.

---

## 📌 Endpoints

### 👤 Usuarios

| Método | Ruta                         | Descripción                    |
|--------|------------------------------|--------------------------------|
| POST   | `/usuarios/`                | Crear nuevo usuario            |
| POST   | `/usuarios/login`           | Login y obtener token JWT      |

---

### ✅ Tareas

| Método | Ruta                          | Descripción                             |
|--------|-------------------------------|-----------------------------------------|
| GET    | `/tareas/`                   | Obtener todas las tareas del usuario    |
| POST   | `/tareas/`                   | Crear una nueva tarea                   |
| DELETE | `/tareas/`                   | Eliminar todas las tareas del usuario   |
| GET    | `/tareas/<id>`              | Obtener una tarea por ID                |
| PUT    | `/tareas/<id>`              | Cambiar el estado de una tarea          |
| DELETE | `/tareas/<id>`              | Eliminar una tarea por ID               |

---

## 🛠️ Tecnologías

- **Rust** 🦀
- **Rocket.rs** 🚀
- **RethinkDB** 🔁
- **JWT (jsonwebtoken)** 🔐
- **WebSocket** (Rocket) 📡
- **Swagger** con [`utoipa`] 📘

---

## 📄 Licencia

Este proyecto es **free** y puedes modificarlo y usarlo como desees.

---

## 📬 Contacto

Desarrollado por **Wuilmer Morgado**  
📧 Email: wuilmermorgado24@gmail.com
