# How to config
- Create a file named `.env` in the root directory of the project.
- Add the following content to the `.env` file:
```
DATABASE_URL=`the url of the database,the format is : mysql://username:password@host:port/database`
PERSONAL_GITHUB_TOKEN=`your personal access token of github`
DELAY=`the poll delay time of server, in seconds. e.g. val set 3600 means 1 hour`
PORT=`the port of the server`
ADDRESS=`the listening address of the server`
```