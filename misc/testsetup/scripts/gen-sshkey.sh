
echo ":: generate ssh key"
mkdir -p ${GERRIT_SITE}/.ssh
chmod -R go-rwx ${GERRIT_SITE}/.ssh
ssh-keygen -t rsa -N "" -f ${GERRIT_SITE}/.ssh/id_rsa
chown -R ${GERRIT_USER}:${GERRIT_USER} ${GERRIT_SITE}/.ssh

ln -sf ${GERRIT_SITE}/.ssh ${GERRIT_HOME}/.ssh
chown -R ${GERRIT_USER}:${GERRIT_USER} ${GERRIT_HOME}/.ssh
