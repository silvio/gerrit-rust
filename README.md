
# Introduction

`gerrit-rust` is a console client for gerrit written in rust. This is a **rust
learner** project. Have patience with me :-)  
I'm happy about every PR, but I will ask questions about changes to learn from
your knowledge.

# Dependency

*   A installed `curl` binary in `$PATH`

# Design

Some design considarations here.

*   [ ] semver at version 1.0.0. Before this version no semver!
*   [x] **0.1.5** Use of curl-rs as http client
*   [x] **0.1.0** Use a config file `.ggr.config` in TOML format

    *   `baseurl`: base url with schema (http)
    *   `port`: used port
    *   `appendix`: for gerrit server under a subpath (not tested)
    *   [x] **0.1.3** User authentication, without `username` and/or `password`
        we use the anonymous backend of gerrit
        *   `username`: username for login
        *   `password`: password for login
    *   `root`: true if this is the uppermost project of all repositories
        underneath
    *   [ ] Authentication (e.g.: digest, basic)
        *   [x] only `digest` hardcoded supported

    *   Consider to configure via config file or put all settings into
        as entries in `.git/config`.  
        The values are same for config file and git-config approach. For the
        git-config we use `ggr-` as a prefix.

    *   provide a configuration frontend

        *   [ ] `ggr config set baseurl 'http://localhost'`: set new `baseurl`
        *   [ ] `ggr config unset -C project1 root`: remove `root` in project1
            repository
        *   [ ] `ggr config set root`: set root for current repository
        *   [x] **0.1.0** `ggr config list`: list all options
            *   [ ] ... with origin of setting
        *   [ ] `ggr config generate --base <...> ...`: generates
            a `.ggr.config` file

*   manage of topics over more than one repository (git submodules like)

    *   [x] **0.1.0** `ggr topic create <BRANCHNAME> [-r sub:rev]`  
        Create branch at main folder and specified subfolders. If a branch with
        same name exists it isn't touched. `rev` is the reference where branch
        should created Defaults to `orign/master`. For base folder use `-r .`.

        *   [ ] execute in subfolders: create branch in this repo and in
                baserepo.
        *   [ ] Add `-R` option to create branch on base and all
                subrepositories.

    *   [x] **0.1.0** `ggr topic forget <BRANCHNAME> [-R]`  
        Delete a branch at mainfolder and and with `-R` in all subfolders.

        *   [ ] Add option `-s` in conjunction with `-R` to remove all branches
            recursive which have no commit and the repositories are clean. Warn
            unclean repositories/branches.

    *   [ ] `ggr topic list -s`  
        List all development branches and the repositories. With `-s` it
        includes the commits in the branch like `git submodule summary`.

    *   [ ] `ggr topic checkout <BRANCHNAME>`  
        checkout a branch.

    *   [ ] `ggr topic push [-b] [<BRANCHNAME>]`  
        Push changes to gerrit. Without `-b` its pushed to gerrit. With option
        `-b` its pushed to a build server. Without branchname the current
        branch is pushed.

    *   [ ] `ggr topic pull <BRANCHNAME>`  
        Pulls a topic.

    *   [ ] `ggr topic reviewer [<BRANCHNAME>] [-r <MAIL>] [-c <MAIL>] [-t <MAIL>]`  
        Add reviewer (`-r`), CC: (`-c`) or TO: (`-t`) at topic push time. The
        information is put to branch config like `config.BRANCHNAME.ggr-re
        MAIL`. Config lineentries start with ggr-\[cc/to/re\]. Without any
        options the current reviewer/to/cc showed for current branch. Without
        BRANCHNAME te current branch is taken.

*   Other Ideas

    *   [x] **0.1.0** `ggr changes query <QUERY>`
        query a searchstring to gerrit server. Use as `QUERY` the same syntax
        as in gerrit web frontend. eg

        *   [x] **0.1.4** Add `--fields` to get only needed fields back. A talk
            on #rust-de suggested some solutions how to handle the input string
            field names with the ChangesInfo struct. Eg: <https://is.gd/PADslX>

            The current implementation use the parsed json string and give back
            the values from this map.

        *   [x] **0.1.6** Add a `--human` option to print it in human readable
            format.

        *   [ ] Add `--header` to get a header line with field names

        *   [x] **0.1.4** Add `--field-list` to get all selectable fields,
            usable for `--fields` option on a second call.

        *   [x] **0.1.4** Add `--raw` for json in raw format. Usable for pretty
            printer over pipe

        *   [x] **0.1.4** Option `-o`/`--ofields` to get additional information
            of changes back (like REVISION etc ...)

        Examples:

        *   `ggr changes query status:open is:watched n:2`: query open changes
        which `watched` flag.

    *   [ ] create a helper script for setup of development environment

        *   [x] docker based gerrit server  
            found docker image `docker pull docker.io/fabric8/gerrit:latest`
        *   [ ] setup password and username for gerrit
        *   [ ] autogenerate git repositrories and submodules
        *   [ ] setup gerrit for this repositories
        *   [ ] auto push master branches to gerrit

    *   [ ] `ggr stat [-F <date>] [-T <data>]`  
        some statistics like opened and closed review since a week or between
        a timespan. via iso-8601 like `date -Is`.

        Examples:

        *   `ggr stat -F 1w`: last week to now
        *   `ggr stat -F 2015-12-31 -T 2016-02-01`: from 01.01.2016T00:00 till 01.02.2016T23:59:59
        *   `ggr stat -F 2016-01-01`: from 02.01.2016T00:00 till now
        *   `ggr stat -T 2016-02-01`: from begin of gerrit usage till 01.02.2016T23:59:59

    *   reviewer per commit
    *   reviewer per repository

    *   status of branches  
        shows status of a branch (remote and local like `git remote show ...`

        *   `ggr status [<BRANCHNAME>]`

    *   Support for `.repo` folder

    *   Consider to use https://github.com/gsingh93/trace
    *   Consider to use https://github.com/ticki/termion

    *   Document `gerritlib::call` module

    *   [x] **0.1.5** Add .travis.yml

# gerrit demo server local on your host via docker

This creates a dockercontainer which is connectable via http://localhost:8080.
The server is setup for development and all accounts can do all things.
It generate or use a `DOCKER-FOR-GERRIT` folder with all settings, repositories
and ssh-keys.

```text
docker run --rm -it -p 0.0.0.0:8080:8080 -p 127.0.0.1:29418:29418 \
           -e AUTH_TYPE='OpenID' \
           -e GERRIT_PUBLIC_KEYS_PATH='/home/gerrit/ssh-keys' \
           -v ${PWD}/DOCKER-FOR-GERRIT/ssh-keys:/home/gerrit/ssh-keys \
           -v ${PWD}/DOCKER-FOR-GERRIT/site:/home/gerrit/site \
           --name gerrit docker.io/fabric8/gerrit:latest
```


# Workflow

* work on a new branch: `ggr topic create testfeature -F p1:origin/master -F p2`
* hack hack hack
* hmm `p3` needs changes too
* `cd p3 ; ggr topic create testfeature`
* hack hack hack
* needs buildserver for integration test `ggr topic push -b testfeature`
* ssh and do build things
* push changes to gerrit `ggr topic push testfeature`

# Random Notes

## curl

With this we can handle the rest api ...

```text
curl -x GET 'http://localhost:8080/projects/?&b=master'
```

## useful links

* gerrit api documentation: <https://gerrit-review.googlesource.com/Documentation/rest-api.html>
* request, response design from: <https://github.com/gefolderstsentry/sentry-cli>

# License

Licensed under either of

*   Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
*   MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
