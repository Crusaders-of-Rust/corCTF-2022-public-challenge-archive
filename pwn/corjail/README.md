Solves: 1

Author: D3v17

Description:

Containerized environments are no longer a safe place.
Evil hackers continue to refine their secret techniques to bypass modern kernel protectons.
CoRJail, as part of CoROS, is designed to stop them!

With CoRJail, several dangerous syscalls, like msgsnd/msgrcv, are blocked by custom seccomp filters.
Syscall usage is constantly monitored with CoRMon, so that kernel exploit patterns can rapidly be detected.

Try the default CoRMon filter with `cat /proc_rw/cormon` and monitor syscall usage like a boss!
Still not satisfied? Set a custom filter with `echo -n 'sys_msgsnd,sys_msgrcv' > /proc_rw/cormon`.

Wanna access all the other CoROS features? Buy a CoR SaaS License for only $31337.00/mo!
Hackers' days are numbered!


Flag: `corctf{C0R_J4!L_H@S_B33N_PWN3D_991cd43a402cda6c}`
