# Required
* `username` of the gitea instance that repositories are being migrated to
* `password` of the gitea instance that repositories are being migrated to
* `examplegitea.com` root url of the gitea instance
* `username/org name` of the github user to migrate repositories from
* `GitHub password` - **required only if**  `--private`, `--both` or `--all` flags have been set. If no option has been set the default is `--public` which will not require the Github password.

# Flags
* `--public` *(default)* - migrate only all public repositories
* `or --private` - migrate only all private repositories
* `or --both` - migrate only private and public repositories created by that user
* `or --all` - migrate all repositories created by or associated with that user




