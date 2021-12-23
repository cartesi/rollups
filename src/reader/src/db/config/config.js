require("dotenv").config();
let LOGGING;

if (process.env.DB_LOGGING === '1') {
  LOGGING = console.log;
} else {
  LOGGING = false;
}

console.log('host:', process.env.DB_HOST);

module.exports = {
  development: {
    username: process.env.DB_USER,
    password: process.env.DB_PASSWORD,
    database: process.env.DB_NAME,
    host:  "127.0.0.1",
    dialect: "postgres",
    logging: LOGGING,
  },
  test: {
    username: process.env.DB_TEST_USER,
    password: process.env.DB_TEST_PASSWORD,
    database: process.env.DB_TEST_NAME,
    host: "127.0.0.1",
    dialect: "postgres"
  },
  production: {
    username: "root",
    password: null,
    database: "database_production",
    host: "127.0.0.1",
    dialect: "mysql"
  }
};
