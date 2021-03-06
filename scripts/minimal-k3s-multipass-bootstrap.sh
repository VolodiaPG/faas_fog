#!/bin/bash

master="${master:-master}"
# nodes=("node1" "node2")
nodes=()
context="k3s-cluster"

createInstance() {
	multipass launch -n "$1" -d 20G -c 2 -m 4G --cloud-init - <<EOF
users:
- name: ${USER}
  groups: sudo
  sudo: ALL=(ALL) NOPASSWD:ALL
  ssh_authorized_keys:
  - $(cat "${PUBLIC_SSH_KEY_PATH}")
EOF
}

getNodeIP() {
	multipass list | grep "$1" | awk '{print $3}'
}

installK3sMasterNode() {
	MASTER_IP=$(getNodeIP "${1}")
	k3sup install --ip "${MASTER_IP}" --context "${context}" --user "${USER}" --ssh-key "${PRIVATE_SSH_KEY_PATH}"
}

installK3sWorkerNode() {
	NODE_IP=$(getNodeIP "${1}")
	k3sup join --server-ip "$'MASTER_IP'" --ip "${NODE_IP}" --user "${USER}" --ssh-key "${PRIVATE_SSH_KEY_PATH}"
}

createInstance "${master}"

for node in "${nodes[@]}"; do
	createInstance "${node}"
done

installK3sMasterNode "${master}"

for node in "${nodes[@]}"; do
	installK3sWorkerNode "${node}"
done
