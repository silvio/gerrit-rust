#!/usr/bin/env bash
# vim: set sw=4 expandtab

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")

WORKDIR="${WORKDIR:-GERRIT}"
SITE="${SCRIPT_DIR}/${WORKDIR}"
DOCKERIMAGE="${DOCKERIMAGE:-openfrontier/gerrit:latest}"


cleanup() {
    cd ${SCRIPT_DIR}
    docker stop gerritlatesttest
    docker rm gerritlatesttest
    rm -rf "${WORKDIR}"
    rm -rf testsetup/masterproject
}

wait_print() {
    declare -i secs
    declare -i orig_secs

    secs=${1:-60}
    orig_secs=${secs}

    sleep 3
    while ! [ -e ${SITE}/.started ] && [ $secs -gt 0 ]; do
        echo -n "."
        secs=$(( secs-1 ))
        sleep 1
    done
    echo ""

    if [ $secs -le 0 ]; then
        echo ":: run into timeout (${orig_secs})"
        echo ":: following failures possible because this timeout failure"
    else
        secs=$(( orig_secs - secs))
        echo ":: ssh accessible after ${secs}s"
    fi
}

start_gerrit() {
    docker run -h localhost -d \
        --name gerritlatesttest \
        -v ${SITE}:/var/gerrit/review_site \
        -v ${SCRIPT_DIR}/testsetup/scripts/gen-sshkey.sh:/docker-entrypoint-init.d/gen-sshkey.sh \
        -v ${SCRIPT_DIR}/testsetup/scripts/check-running-service.sh.nohup:/docker-entrypoint-init.d/check-running-service.sh.nohup \
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
    git init
    git config --file .git/config user.email "test@example.com"
    git config --file .git/config user.name "gerrit-rust test"
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

echo ":: pull docker image"
docker pull "${DOCKERIMAGE}"

SECS=$((1 * 60))
echo ":: start gerrit server - timeout ${SECS} seconds"
start_gerrit &
wait_print "${SECS}"

echo ":: create masterproject"
ssh -p 29418 -o NoHostAuthenticationForLocalhost=yes  -i ${SITE}/.ssh/id_rsa admin@localhost gerrit create-project masterproject

echo ":: init repositories"
init_repository
