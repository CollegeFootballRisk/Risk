# Risk
A Risk game similar to CollegeFootballRisk, written in Rust

- [API](/documentation/API.md)
- [Deviations from College Football Risk's API](/documentation/DEVIATIONS.md)

# Installation
This programme requires Rust nightly. It can be installed with the following:

```bash curl -s https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly```

Once nightly is installed, you can clone the following repository:

```git clone https://github.com/kennroo/Risk.git```

You'll now need to configure the environmental variables. Use a text editor to edit the following file:

server/.env
	 ```
	 DATABASE_URL=postgresql://{user}@{host}/{database_name}
	 ```

We can now run the diesel migrations. To install diesel, run the following:

`cargo install diesel_cli`

Then enter the directory, create the database and tables with the following:
```
cd Risk/server/
diesel setup
diesel migration run
```
The programme should be able to start up successfully.

`cargo run`
>Error?
>If cargo throws an error about schema, copy SchemaWithViews to schema like so:
`cp src/SchemaWithViews.rs src/schema.rs`
