[![Build Status](https://travis-ci.org/silvio/gerrit-rust.svg?branch=master)](https://travis-ci.org/silvio/gerrit-rust)

# Introduction

`gerrit-rust` is a console client for gerrit written in rust. This is a **rust
learner** project. Have patience with me :-)  
I'm happy about every PR, but I will ask questions about changes to learn from
your knowledge.

# Dependency

*   A installed `git` binary in `$PATH`
*   gerrit server with installed download-plugin

# Design

Some design considarations here.

*   [ ] semver at version 1.0.0. Before this version no semver!
*   remove external depency to host
    *   [ ] git binary
    *   [ ] gerrit with download plugin
*   [x] **0.1.5** Use of curl-rs as http client
*   [x] **0.1.0** Use a config file `.ggr.config` in TOML format

    *   `api`: base url with schema (http)
    *   [x] **0.1.3** User authentication (**deprecated since 0.1.9**)
        *   `username`: username for login
        *   `password`: password for login
        [x] **0.1.9** only `.netrc` settings are respected for username and
            password. u/p in config file are ignored
    *   `root`: true if this is the uppermost project of all repositories
        underneath
    *   [x] Authentication (e.g.: digest, basic)
        *   [x] `digest` and `basic` are supported. Current implementation
            calls both. First one is `basic` and second one is `digest`.

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

    *   [x] **0.1.9** Use `$HOME/.netrc` file to get username and password.

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

    *   [x] **0.1.8** `ggr topic checkout <BRANCHNAME>`  
        Checkout a branch on all repositories.
        *   [x] **0.1.8** first check base folder if checkout happened sync all
            submodules than checkout changes for topic on subfolders

    *   [ ] `ggr topic push [-b] [<BRANCHNAME>]`  
        Push changes to gerrit. Without `-b` its pushed to gerrit. With option
        `-b` its pushed to a build server. Without branchname the current
        branch is pushed.

    *   [x] **0.1.7** <s>`ggr topic pull ...`</s>  
        same as `ggr topic fetch`. This sub function is renamed because `fetch`
        is more in line with git speak than `pull`.

    *   [x] **0.1.8** `ggr topic fetch [-f] [-b branchname] <topicname>`  
        fetch latest version of commits for a topic. Create for all
        changes a branch with the patch identifier as name, or with `-b` with
        a given branchname.
        * [x] **0.1.9** Add tracking information via `--track <branch>` option.
        * [x] **0.1.14** Add `--closed` option to pull closed (merged) topics

    *   [x] **0.1.17** `ggr topic reviewer [<TOPIC>] [-r <+/-MAIL>,...]`  
        Add(+) or remove(-) reviewer (`-r`) from topic. Without an option we
        receive a list of all reviewers on this topic.
        * [x] **0.1.17** `-v`/`--verbose` for detailed view of approvals
        * [ ] Without TOPIC it used the actual topic (!=master) on base and
              submodules.
        * [ ] Add `--format` option for formating of output. Using of rust
              variable and formating informations like `{email}`,
              `{email:15.2}`.

    *   [x] **0.1.18** `ggr topic abandon|restore [<TOPIC>]`  
        Abandon/restore a complete topic.
        *   [ ] Without TOPIC it uses the actual topic
        *   [x] **0.1.18** `[-m <MESSAGE>]` adds a abandon message to all
            changes in this topic
        *   [ ] `[-n <NONE|OWNER|REVIEWERS|ALL>]` notifiy a group of accounts
            about this abandon action or don't notify (via `NONE`). Default is
            `ALL`.

    *   [ ] `ggr topic rename OLDTOPIC TOPIC`  
        Rename OLDTOPIC to TOPIC
        *   [ ] make OLDTOPIC optional, the current topic is renamed

    *   [ ] `ggr topic verify [<TOPICNAME>] <LABEL> [<MESSAGE>]`
        Verify all commits of a topic TOPICNAME with a label LABEL
        (-2|-1|0|+1|+2|=) and a optional message. Be aware the messages is
        appended as note on ALL commits in this topic.
        *   [ ] Label `=` means not changing the current review label value,
            only a append a message

* Query changes

    *   [x] **0.1.0** `ggr changes query <QUERY>`
        query a searchstring to gerrit server. Use as `QUERY` the same syntax
        as in gerrit web frontend. eg

        *   [x] **0.1.7** Add `--regexp-selector` to show only keys selected by
            regular expression.
            This remove the --fields selector introduced in 0.1.4.

        *   [x] **0.1.6** Add a `--human` option to print it in human readable
            format.

        *   [x] **0.1.4** Add `--field-list` to get all selectable fields,
            usable for `--fields` option on a second call.

        *   [x] **0.1.4** Add `--raw` for json in raw format. Usable for pretty
            printer over pipe

        *   [x] **0.1.4** Option `-o`/`--ofields` to get additional information
            of changes back (like REVISION etc ...)

        Examples:

        *   `ggr changes query status:open is:watched n:2`: query open changes
        which `watched` flag.

* Library features

    *   [x] **0.1.16** cli needs a subcommand to do lowlevel task -> gerritapi

    *   [x] **0.1.0** implement base for http requests and responses

    *   [ ] build a feature complete library to work with gerrit servers

        *   [ ] access endpoint
        *   [ ] accounts endpoint
        *   [ ] changes endpoint
            *   [x] **0.2.0** Create change
            *   [x] **0.2.0** Query Changes
            *   [ ] Get Change
            *   [ ] Get Change Detail
            *   [ ] Get Topic
            *   [ ] Set Topic
            *   [ ] Delete Topic
            *   [ ] Abandon Change
            *   [x] **0.2.2** Restore Change
            *   [x] **0.2.2** Rebase Change
            *   [ ] Move Change
            *   [ ] Revert Change
            *   [ ] Submit Change
            *   [ ] Changes Submitted Together
            *   [ ] Publish Draft Change
            *   [ ] Delete Draft Change
            *   [ ] Get Included In
            *   [ ] Index Change
            *   [ ] List Change Comments
            *   [ ] List Change Drafts
            *   [ ] Check Change
            *   [ ] Fix Change
        *   [ ] reviewer endpoint
            *   [x] **0.2.1** List Reviewers
            *   [ ] Suggest Reviewers
            *   [x] **0.2.1** Get Reviewer
            *   [x] **0.2.1** Add Reviewer
            *   [x] **0.2.1** Delete Reviewer
            *   [ ] List Votes
            *   [ ] Delete Vote
        *   [ ] Revision Endpoints
            *   [ ] Get Commit
            *   [ ] Get Revision Actions
            *   [ ] Get Review
            *   [ ] Get Related Changes
            *   [ ] Set Review
            *   [ ] Rebase Revision
            *   [ ] Submit Revision
            *   [ ] Publish Draft Revision
            *   [ ] Delete Draft Revision
            *   [ ] Get Patch
            *   [ ] Get Mergeable
            *   [ ] Get Submit Type
            *   [ ] Test Submit Type
            *   [ ] Test Submit Rule
            *   [ ] List Revision Drafts
            *   [ ] Create Draft
            *   [ ] Get Draft
            *   [ ] Update Draft
            *   [ ] Delete Draft
            *   [ ] List Revision Comments
            *   [ ] Get Comment
            *   [ ] List Files
            *   [ ] Get Content
            *   [ ] Download Content
            *   [ ] Get Diff
            *   [ ] Get Blame
            *   [ ] Set Reviewed
            *   [ ] Delete Reviewed
            *   [ ] Cherry Pick Revision
        *   [ ] config endpoint
            *   [x] **0.2.0** Get Version
            *   [ ] Get Server Info
            *   [ ] Confirm Email
            *   [ ] List Caches
            *   [ ] Cache Operations
            *   [ ] Get Cache
            *   [ ] Flush Cache
            *   [ ] Get Summary
            *   [ ] List Capabilities
            *   [ ] List Tasks
            *   [ ] Get Task
            *   [ ] Delete Task
            *   [ ] Get Top Menus
            *   [ ] Get Default User Preferences
            *   [ ] Set Default User Preferences
            *   [ ] Get Default Diff Preferences
            *   [ ] Set Default Diff Preferences
        *   [ ] groups endpoint
        *   [ ] plugins endpoint
        *   [ ] projects endpoint

*   Other Ideas

    *   [x] **0.1.9** implement a log mechanism to get debugging information
        via loglevel switch

    *   [x] **0.1.16** do work to support more than one gerrit server

    *   [x] **0.1.14** create a helper script for setup of development
        environment

        *   [x] docker based gerrit server  
            found docker image `docker pull openfrontier/gerrit`
        *   [x] setup password and username for gerrit
        *   [x] autogenerate git repositrories and submodules
        *   [x] setup gerrit for this repositories
        *   [x] auto push master branches to gerrit

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

    *   [x] **0.1.11** Document `gerritlib::call` module

    *   [x] **0.1.5** Add .travis.yml

# gerrit demo server local on your host via docker

This creates a dockercontainer which is connectable via http://localhost:8080.
The server is setup for development and all accounts can do all things.
It generate or use a `DOCKER-FOR-GERRIT` folder containing of settings,
repositories and ssh-keys.

```text
docker run --rm -it \
           -h localhost
           -p 8080:8080 -p 29418:29418 \
           -v /development/projects/DOCKER-FOR-GERRIT:/var/gerrit/review_site \
           --name gerrit \
           openfrontier/gerrit:latest
```


## useful links

* gerrit api documentation: <https://gerrit-review.googlesource.com/Documentation/rest-api.html>
* request, response design from: <https://github.com/gefolderstsentry/sentry-cli>

# License

Licensed under

*   Mozilla Public License 2.0 ([LICENSE-MPL-2.0](LICENSE-MPL-2.0) or https://www.mozilla.org/media/MPL/2.0/index.txt)

