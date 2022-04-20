# Rollups data


## Perform database migrations

```shell
$ cargo install diesel_cli --no-default-features --features postgres
$ export DATABASE_URL=postgres://<username>:<password>@<database_host>:<database_port>/<database_name>
$ diesel migration run
```