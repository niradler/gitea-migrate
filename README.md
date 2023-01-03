# gitea-migrate
Migrate or mirror multiple repositories from Github to a Gitea instance.
It can be used manually as a cli tool, or automated on a server with something like cron jobs. The main aim of this repository was to learn more rust and publish code to [crates.io](crates.io)!


# Features
Specify exactly which repositories should be mirrored or migrated with the following flags:
* `--public` - (default) only mirror all public repositories under your username
* `--private` - only mirror all private repositories under your username
* `--both` - only mirror all private and public repositories under your username
* `--all` - mirror all private and public repositories including ones from other users that you are associated with
* `--no-mirror` - will only clone the repositories (not as mirrors)
* `--fork` - will include forked repositories

# Usage
You must specify a destination URL to the target Gitea instance as a cli argument.
Since migrating repositories requires user authorization, you will be prompted to enter your github 
username and access [token](https://github.com/settings/tokens). Additionally, you will be required 
enter the username and password for your target Gitea instance. However, if you don't wish to manually
enter credentials every time, you can simply enter your credentials in a file and specifiy with the 
`--creds` flag the name of the file.

`credentials.txt`
``` txt
github_username:github_access_token
gitea_username:gitea_password
```
**IMPORTANT** Be sure to follow the above layout exactly, otherwise your credentials won't be parsed correctly. (if you feel like there is a better way to store credentials open a [pull request](https://github.com/maxgallup/gitea-migrate/pulls)! )

### Examples
```
gitea-migrate --dest https://gitea.example.com --both
```
This will **mirror** all private and public repos under the github username that was manually entered.

```
gitea-migrate --dest https://gitea.example.com --all --no-mirror --creds ~/credentials.txt 
```
This will **migrate** (and not mirror) all private and public repos associated with the user specified in the `credentials.txt` file. 

### Install
```
cargo install gitea-migrate
```



# TODOs
- [ ] add logging
- [ ] add support for other git platforms (gitlab, bitbucket...)
- [ ] add async code or multithreading? - overkill for this project, but the purpose of this project is to learn anyway ;)
- [ ] improve maintainability of source code

