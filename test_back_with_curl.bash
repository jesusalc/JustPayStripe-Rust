curl -X POST http://localhost:8081/customer \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Rust Test",
    "email": "rust@test.com",
    "phone": "333-333-3333",
    "description": "A test customer from rust."
  }'
