# Risk
A Risk game similar to CollegeFootballRisk, written in Rust

- [API](/documentation/API.md)
- [Deviations from College Football Risk's API](/documentation/DEVIATIONS.md)

# Installation
This programme requires Rust nightly. It can be installed with [rustup](https://github.com/rust-lang/rustup#choosing-where-to-install).

Once nightly is installed, you can clone the following repository:

```
git clone https://github.com/kennroo/Risk.git
```

You'll now need to configure the environmental variables. Use a text editor to edit the following file:

**server/.env**

```
DATABASE_URL=postgresql://{user}@{host}/{database_name}
```

We can now create the database. Use postgres to [Create a Database](https://www.postgresql.org/docs/9.0/sql-createdatabase.html).

Once a database has been created, you will need to run the **db/up.sql** file using the following:
```
psql -U {user} -f db/up.sql
```

Next, copy sample.env to .env:
```
cp sample.env .env
```

Edit the .env file with your reddit and/or discord keys. Similarly edit the uri and other information. The next file to edit is **server/Rocket.toml**. A sample one follows:
``` 
[global.oauth.reddit]
provider = "Reddit"
client_id = ""
client_secret = ""
redirect_uri = "http://localhost/auth/reddit"
[staging]
address = ""
port = 
keep_alive = 5
log = ""
limits = { forms = 32768 }
[production]
address=""
port=
keep_alive=5
log = ""
limits = { forms = 32768 }
secret_key = ""
```

The server programme should now be able to start up successfully, but does not have any data.

`cargo run`
>Error?
>If cargo throws an error about schema, copy SchemaWithViews to schema like so:
`cp src/SchemaWithViews.rs src/schema.rs`


# Contributing

## Ringmaster

*Ringmaster* is the programme routine which determines territory ownership, MVPs, and statistics for each turn. It runs on a chron job each night. 

![Ringmaster Flamegraph](/documentation/flamegraph.svg)
Produced with [FlameGraph](https://github.com/flamegraph-rs/flamegraph).
> See the current [projects](https://github.com/mautamu/Risk/projects). Feel free to submit a pull request, and include a new flamegraph. 


## Server


*Server* is the programme routine which serves both static and dynamic content to end users. If you are experiencing an issue accessing data on the website, it is highly likely that the *Server* programme is to blame. If you would like to improve the design elements, feel encouraged. Adding functionality by pull request is similarly appreciated, but check the projects or submit a PR before undertaking the project.


> See the current [projects](https://github.com/mautamu/Risk/projects). 
