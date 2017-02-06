#!/usr/bin/env bash
# vim: set sw=4 expandtab

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")

SITENAME="GERRIT"
SITE="${SCRIPT_DIR}/${SITENAME}"
DOCKERIMAGE="silviof/docker-gerrit:no-check-for-dotfiles"


cleanup() {
    cd ${SCRIPT_DIR}
    docker stop gerritlatesttest
    docker rm gerritlatesttest
    rm -rf "${SITENAME}"
    rm -rf testsetup/masterproject
}

wait_print() {
    declare -i secs
    secs=${1:-60}

    while [ $secs -gt 0 ]; do
        echo -n "."
        sleep 1
        : $((secs--))
    done
    echo ""
}

create_ssh() {
    mkdir -p ${SITE}/.ssh
    chmod -R go-rwx ${SITE}/.ssh
    ssh-keygen -t rsa -N "" -f ${SITE}/.ssh/id_rsa
}

start_gerrit() {
    docker run -h localhost -d \
        --name gerritlatesttest \
        -v ${SITE}:/var/gerrit/review_site \
        -e AUTH_TYPE=DEVELOPMENT_BECOME_ANY_ACCOUNT \
        -e GERRIT_INIT_ARGS='--install-plugin=download-commands' \
        -p 8080:8080 -p 29418:29418 \
        "${DOCKERIMAGE}" > /dev/null
    echo -n "D"
}

init_repository() {
    mkdir -p ${SCRIPT_DIR}/testsetup/masterproject
    cp ${SCRIPT_DIR}/testsetup/masterproject-sources/lorem-ipsum.txt ${SCRIPT_DIR}/testsetup/masterproject/
    cp ${SCRIPT_DIR}/.ggr.conf ${SCRIPT_DIR}/testsetup/masterproject

    cd ${SCRIPT_DIR}/testsetup/masterproject
    git config user.email "test@example.com"
    git config user.name "gerrit-rust test"
    git init
    curl -Lo .git/hooks/commit-msg http://localhost:8080/tools/hooks/commit-msg
    chmod u+x .git/hooks/commit-msg
    git add lorem-ipsum.txt
    git commit -m "initial commit" --no-edit
    git remote add origin http://admin:secret@localhost:8080/masterproject
    git push -u origin master
    git checkout -b testbranch --track origin/master
    git am ${SCRIPT_DIR}/testsetup/masterproject-sources/*.patch
    git push origin testbranch:refs/for/master/testbranch
}

echo ":: cleanup"
cleanup

echo ":: ssh key creation"
create_ssh

echo ":: pull docker image"
docker pull "${DOCKERIMAGE}"

SECS=$((1 * 40))
echo ":: start gerrit server - wait ${SECS} seconds"
start_gerrit &
wait_print "${SECS}"

echo ":: create masterproject"
ssh -p 29418 -o NoHostAuthenticationForLocalhost=yes  -i ${SITE}/.ssh/id_rsa admin@localhost gerrit create-project masterproject

echo ":: init repositories"
init_repository
