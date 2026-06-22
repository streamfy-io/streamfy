---
name: Bug report
about: Create a report to help us improve
title: "[Bug]:"
labels: bug
assignees: ''

---

**What happened**
A clear and concise description of what the bug is.

**Expected behavior**
A clear and concise description of what you expected to happen.

**Describe the setup**
- Are you using a local Streamfy install? Minikube? Streamfy Cloud?
- What version of Streamfy are you using? `streamfy version`

**How to reproduce it (as minimally and precisely as possible)**
Steps to reproduce the behavior:
1. Run the command '...'
2. Type the input '...'

**Log output**
It helps to have logs from Streamfy's SC and SPU processes.
Depending on your setup, here's how you can get the logs:

- For a local Streamfy installation on Mac & Linux:
  - Run `cat ~/.streamfy/log/flv_sc.log` for SC logs
  - Run `cat ~/.streamfy/log//spu_log_XXXX.log` for each SPU
    - E.g. when running 1 SPU, there will be `spu_log_5001.log`
- For a Streamfy installation on Minikube:
  - Run `kubectl logs streamfy-sc` for SC logs
  - Run `kubectl logs streamfy-spg-main-X` for each SPU

**Environment (please complete the following information):**
- OS: [e.g. Linux, Mac]
- Streamfy Version [e.g. 22]
- Kubernetes version: use `kubectl version`
- Minikube or other k8 version (if used): use `minikube version`

**Optional Debugging:**
Another trace which may be useful is re-running the command with RUST_LOG specified at a higher level
`RUST_LOG=info streamfy ...` or `RUST_LOG=debug streamfy ...`

**Additional context**
Add any other context about the problem here.
