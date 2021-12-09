# rs115

```
Yves Lelouch <@jkb_uhi>
Quick actions to use on 115.com

USAGE:
    rs115 [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    check          use this command to check if the name is allowed for uploading
    clean          clean up login info
    help           Prints this message or the help of the given subcommand(s)
    set-cookies    set cookies to login 115.com
    status         print status
```

## Usage:

1. To login, set your cookies from 115.com:

```
./rs115 set-cookies "*******************COOKIES*********************"
```

2. To verify login status:

```
./rs115 status
```

3. To check if a name is valid, for example "github"

```

./rs115 check github

```

4. To check many names at once, create a text file where your names to check listed line by line.

```
./rs115 check -f <path to your checklist>
```

you can log the invalid cases to a file by using `-o` flag, the failed case by using `-x` flag.

```
./rs115 check -f <checklist> -o <file of invalid names> -x <file of failed cases>
```

5. To clean your login session, use clean subcommand:

```
./rs115 clean
```
