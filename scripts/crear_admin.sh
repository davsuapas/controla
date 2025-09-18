curl -i -X POST http://localhost:8080/api/usuarios \
  -H "Content-Type: application/json" \
  -d '{
    "id": 0,
    "autor": 0,
    "dni": "",
    "email": "",
    "nombre": "",
    "primer_apellido": "",
    "segundo_apellido": "",
    "password": "",
    "activo": "2025-09-18T10:30:00",
    "inicio": null,
    "roles": []
  }'