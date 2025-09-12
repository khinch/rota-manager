# rota-manager
LGR Accelerator Capstone Project - Application to manage nursery staff/child rota: times, ratio and requirements.

# Running Locally
## Initialising PostgreSQL
``` shell
docker pull postgres:15.2-alpine
docker run --name lgr-ps-db -e POSTGRES_PASSWORD=[POSTGRES_PASSWORD] -p 5432:5432 -d postgres:15.2-alpine
```

## Initialising Redis
``` shell
docker run --name lgr-redis-db -p "6379:6379" -d redis:7.0-alpine
```
