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

## Logs
Default log level is INFO. To set a specific log level, set the `RUST_LOG` environment variable, e.g.:
``` shell
RUST_LOG=<level>
```
where level is one of: `ERROR | WARN | INFO | DEBUG | TRACE`
