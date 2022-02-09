# Reader Node

## Description
The Reader Node allows frontend clients to read data about their transactions running in the Cartesi machine

## Getting Started
---
### Using the docker container [Recommended]
* Install docker and docker-compose on your machine [here](https://docs.docker.com/compose/gettingstarted/)

* Define the environment variables:
  * rename `.env.example` to `.env`
  * Add the correct values to the `.env` file
  ```shell 
    DB_USER=database_user
    DB_NAME=database_name
    DB_HOST=database_host
    QUERY_CONTAINER_VERSION=query_container_version
    ```
  * Install dependencies by running `yarn install`
  * Build container image `docker-compose build`
  * Migrate schema to database `docker-compose run --rm app sequelize db:migrate`
  * Seed all tables `docker-compose run --rm app sequelize db:seed:all`
  * Run docker container `docker-compose up`
  
---
### Without docker container
- You will need to have the sequelize-cli installed to be able to run migrations and seed data into your database. Yuo can run the following to install the CLI:

``` bash
$ npm install -g sequelize-cli
```
or

``` bash
$ yarn global add sequelize-cli
```

- This project uses an SQL database and particularly PostgrSQL. You can download it [here](https://www.postgresql.org/download/) or use an online PostgreSQL database provider.
---
 1. **Install the project's dependencies**
Run the following commands in your terminal to install the dependencies for this project:
``` bash
$ npm install
```
or

``` bash
$ yarn install
```

 2.  **Define Environment Variables**
Once you have all packages installed, create a `.env` in the root folder of the project. You should then specify the appropriate environment variable as so:

```env
DB_USER='Your database username'
DB_PASSWORD='Your database user's password'
DB_NAME='Your database name'
DB_TEST_USER='Your test database username'
DB_TEST_PASSWORD='Your test database user's password'
DB_TEST_NAME=DB_NAME='Your test database name'
```
`NB: Ignore the quotes in the fields.`

3.  **Database Migrations**
Run the following command in your terminal to migrate the Model structures to your database:

```bash
$ npm run migrate
```
or
```bash
$ yarn run migrate
```

4. ### **Seed the Database (Optional)**
Run the following command in your terminal to prefill your database with test data:

```bash
$ npm run seed
```
or
```bash
$ yarn run seed
```

5. ### **Start the Server**
Run the following command to start the development server:

```bash
$ npm start
```
or
```bash
$ yarn start
```
go to [http://localhost:4000/graphql](http://localhost:4000/graphql) to utilize graphql's playground to test the queries and mutations. Sample queries and mutations can be found in the root of the project as `sample-queries.txt` and `sample-mutations.txt` respectively.

---
# Development
## Database
Database related definitions like Models, Migrations, Seeders and configurations are located in the path: `src/db`

## GraphQL
GraphQL resolvers, type definition, generated TypeScript types and schemas are located in the path: `src/graphql`. This project utilizes [Codemon](https://web.codemon.com/) to help generate TypeScript types from graphql schemas to help type resolvers appropriately. When there is a change in the `src/graphql/trpeDefs/typeDefs.graphql` file, run the following command to generate the new additions:

```bash
$ npm run generate-types
```
or
```bash
$ yarn run generate-types
```
The generated types are found in `src/graphql/generated-typeDefs.ts`.

## JoinMonster
This project utilizes [JoinMonster](https://join-monster.readthedocs.io/) to help query the database by generating sql queries from the Query information GraphQL provides. Simply put, it queries the database for only the fields that are needed by a GraphQL client.
JoinMonster requires us to define how data should be fetched and what to return for database associations. All these are done in the file located at: `src/joinMonsterMetadata/index.ts`

## Tests
Tests for this project are located in the path: `src/test`. You can run tests by running the following commands in your terminal:
```bash
npm run test
```
or
```bash
yarn run test
```